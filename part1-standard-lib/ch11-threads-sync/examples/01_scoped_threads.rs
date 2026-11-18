// 스코프 스레드: thread::scope 안에서 만든 스레드는 지역 변수를 move 없이 빌려 쓸 수 있습니다.
// 스코프가 끝나기 전에 모든 스레드가 join된다는 보장이 있기 때문입니다.
//
// 실행: cargo run -p p1-ch11-threads-sync --example 01_scoped_threads
//
// 5번 구역은 일부러 스레드 하나를 패닉시킵니다. 기본 패닉 훅이 표준 에러에 메시지를
// 남기지만(스레드 ID는 실행마다 다름), 예제는 패닉을 붙잡으므로 종료 코드는 0입니다.

use std::panic;
use std::thread;

fn main() {
    println!("1. 스코프 스레드는 지역 변수를 move 없이 빌려 읽습니다");
    {
        let words = vec!["사과", "바나나", "체리", "두리안"];
        let limit = 2;

        // thread::spawn이었다면 words를 빌리는 클로저는 E0373으로 거부됩니다(Part 0 ch15).
        // scope는 이 블록이 끝나기 전에 두 스레드를 모두 join하므로 words가 먼저
        // 사라질 일이 없고, 그래서 &words를 그대로 빌려 줄 수 있습니다.
        thread::scope(|s| {
            let long = s.spawn(|| words.iter().filter(|w| w.chars().count() > limit).count());
            let total = s.spawn(|| words.iter().map(|w| w.chars().count()).sum::<usize>());
            println!(
                "    {limit}글자보다 긴 단어 {}개, 글자 수 합계 {}",
                long.join().unwrap(),
                total.join().unwrap()
            );
        });
        // 스코프가 끝나면 대여도 끝났으므로 words를 다시 마음대로 쓸 수 있습니다.
        println!("    스코프가 끝난 뒤에도 words를 씁니다: {words:?}");
        // 출력: 2글자보다 긴 단어 2개, 글자 수 합계 10
        // 출력: 스코프가 끝난 뒤에도 words를 씁니다: ["사과", "바나나", "체리", "두리안"]
    }

    println!("2. 겹치지 않는 조각이면 여러 스레드가 동시에 가변으로 빌릴 수 있습니다");
    {
        let mut numbers: Vec<u32> = (1..=8).collect();

        thread::scope(|s| {
            // chunks_mut는 서로 겹치지 않는 &mut [u32] 조각을 내줍니다.
            // 조각마다 가변 참조가 하나씩이므로 대여 규칙을 지키면서 동시에 고칩니다.
            // move가 옮기는 것은 조각을 가리키는 &mut 참조이지 데이터가 아닙니다.
            for chunk in numbers.chunks_mut(3) {
                s.spawn(move || {
                    for n in chunk.iter_mut() {
                        *n *= 10;
                    }
                });
            }
        }); // 여기서 세 스레드가 모두 끝났음이 보장됩니다.

        println!("    {numbers:?}");
        // 출력: [10, 20, 30, 40, 50, 60, 70, 80]
    }

    println!("3. join하지 않은 스레드도 스코프가 끝날 때 자동으로 기다립니다");
    {
        let mut slots = [0u64; 4];

        thread::scope(|s| {
            // 핸들을 버려도(변수에 담지 않아도) 스코프가 끝나기 전에 join됩니다.
            // thread::spawn에서 핸들을 버리면 스레드가 떨어져 나가는 것(detach)과 다릅니다.
            for (i, slot) in slots.iter_mut().enumerate() {
                s.spawn(move || {
                    let start = i as u64 * 25 + 1;
                    *slot = (start..start + 25).sum();
                });
            }
        });

        println!(
            "    조각별 합: {slots:?}, 전체 합: {}",
            slots.iter().sum::<u64>()
        );
        // 출력: 조각별 합: [325, 950, 1575, 2200], 전체 합: 5050
    }

    println!("4. scope는 클로저의 반환값을 그대로 돌려줍니다");
    {
        let text = "가나다 라마 바사아자 차";
        let (min, max) = thread::scope(|s| {
            let shortest = s.spawn(|| text.split(' ').map(|w| w.chars().count()).min());
            let longest = s.spawn(|| text.split(' ').map(|w| w.chars().count()).max());
            (shortest.join().unwrap(), longest.join().unwrap())
        });
        println!("    가장 짧은 단어 {min:?}글자, 가장 긴 단어 {max:?}글자");
        // 출력: 가장 짧은 단어 Some(1)글자, 가장 긴 단어 Some(4)글자
    }

    println!("5. 스코프 안의 패닉");
    {
        let values = [3, 0, 5];

        // (가) 직접 join한 스레드의 패닉은 join이 돌려주는 Err로 받습니다.
        //     이 경우 scope 자체는 패닉하지 않습니다.
        let joined = thread::scope(|s| {
            let handle = s.spawn(|| 100 / values[1]); // 0으로 나눔 → 패닉
            handle.join().is_err()
        });
        println!("    (가) join이 Err를 돌려줬는가: {joined}");
        // 출력: (가) join이 Err를 돌려줬는가: true

        // (나) join하지 않은 스레드가 패닉하면, 스코프는 나머지 스레드를 모두 기다린 뒤
        //     scope 함수 자체가 패닉합니다. 여기서는 그 패닉을 catch_unwind(Part 1 ch8)로
        //     붙잡아 페이로드를 확인합니다.
        let result = panic::catch_unwind(|| {
            thread::scope(|s| {
                s.spawn(|| 100 / values[1]); // 핸들을 버림
                s.spawn(|| 100 / values[2]); // 이 스레드는 정상 종료
            });
        });
        let message = match &result {
            Ok(()) => "패닉 없음".to_string(),
            Err(payload) => payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "알 수 없는 페이로드".to_string()),
        };
        println!(
            "    (나) scope가 패닉했는가: {}, 페이로드: {message}",
            result.is_err()
        );
        // 출력: (나) scope가 패닉했는가: true, 페이로드: a scoped thread panicked
    }
}
