// 스레드에 값을 넘길 때(move 클로저)와 스레드에서 값을 돌려받을 때(JoinHandle<T>)
// 소유권이 어떻게 옮겨 가는지 보여 줍니다.
//
// 실행: cargo run -p p0-ch15-concurrency --example 02_move_and_return
//
// 5번 구역은 일부러 스레드 하나를 패닉시킵니다. 그 패닉 메시지는 표준 에러로 나가며,
// main은 계속 실행되어 정상 종료합니다(종료 코드 0).

use std::thread;

fn main() {
    println!("1. move 클로저로 값의 소유권을 새 스레드에 넘깁니다");
    let v = vec![1, 2, 3];
    // move가 없으면 클로저는 v를 빌리려 하고, 컴파일러는 E0373으로 거부합니다
    // (compile_fail/e0373_thread_borrows_local.rs). 새 스레드가 main의 지역 변수 v보다
    // 오래 살아남을 수 있으니, 빌린 참조가 댕글링 참조가 될 수 있기 때문입니다.
    let handle = thread::spawn(move || {
        // 이제 v의 소유자는 이 클로저(= 새 스레드)입니다.
        println!("    [새 스레드] v = {v:?}");
    }); // 여기서부터 main은 v를 쓸 수 없습니다. 쓰면 E0382입니다.
    handle.join().unwrap();
    // 출력:     [새 스레드] v = [1, 2, 3]

    println!("2. 스레드가 돌려준 값은 join으로 받습니다");
    let names = vec![String::from("하나"), String::from("둘")];
    // 클로저가 돌려주는 값의 타입이 JoinHandle<T>의 T가 됩니다. 여기서는 Vec<String>입니다.
    let handle: thread::JoinHandle<Vec<String>> = thread::spawn(move || {
        let mut names = names; // 넘겨받은 벡터를 가변으로 다시 바인딩합니다.
        names.push(String::from("셋"));
        names // 소유권을 돌려줍니다.
    });
    // join은 Result<T, _>를 돌려줍니다. Ok 안에 스레드가 돌려준 값이 들어 있습니다.
    let names = handle.join().unwrap();
    println!("    돌려받은 벡터: {names:?}");
    // 출력:     돌려받은 벡터: ["하나", "둘", "셋"]

    println!("3. 원본도 계속 써야 하면 복제(clone)해서 넘깁니다");
    let original = vec![10, 20, 30];
    let copy_for_thread = original.clone(); // 스레드에 줄 사본을 따로 만듭니다.
    let handle = thread::spawn(move || copy_for_thread.iter().sum::<i32>());
    let sum = handle.join().unwrap();
    println!("    스레드가 계산한 합: {sum}, main의 원본: {original:?}");
    // 출력:     스레드가 계산한 합: 60, main의 원본: [10, 20, 30]

    println!("4. 일을 나눠 맡기고 결과를 모읍니다");
    let numbers: Vec<u64> = (1..=100).collect();
    let mut handles = Vec::new();
    for (index, chunk) in numbers.chunks(25).enumerate() {
        // chunk는 numbers를 빌린 슬라이스(&[u64])라서 그대로는 스레드에 넘길 수 없습니다.
        // to_vec()으로 스레드가 소유할 Vec을 만들어 넘깁니다.
        // index는 usize(Copy 타입)라서 move하면 복사본이 들어가고, 원래 값도 그대로 남습니다.
        let part = chunk.to_vec();
        handles.push(thread::spawn(move || {
            let partial: u64 = part.iter().sum();
            (index, partial)
        }));
    }
    let mut total = 0;
    // handles를 만든 순서대로 join하므로, 아래 출력 순서는 스레드가 끝난 순서와 상관없이 항상 같습니다.
    for handle in handles {
        let (index, partial) = handle.join().unwrap();
        println!("    {index}번 조각의 합: {partial}");
        total += partial;
    }
    println!("    전체 합: {total}");
    // 출력:     0번 조각의 합: 325
    // 출력:     1번 조각의 합: 950
    // 출력:     2번 조각의 합: 1575
    // 출력:     3번 조각의 합: 2200
    // 출력:     전체 합: 5050

    println!("5. 스레드가 패닉하면 join이 Err를 돌려줍니다");
    let handle = thread::spawn(|| {
        let empty: Vec<i32> = Vec::new();
        // 빈 벡터의 첫 원소를 꺼내려 하므로 이 스레드는 패닉합니다.
        empty[0]
    });
    // 패닉은 그 스레드만 멈춥니다. main은 join의 결과로 패닉 사실을 알게 됩니다.
    // 지금까지 join().unwrap()을 쓴 것은 "스레드가 패닉했다면 main도 멈추겠다"는 뜻이었습니다.
    match handle.join() {
        Ok(value) => println!("    값: {value}"),
        Err(_) => println!("    스레드가 패닉했습니다. main은 계속 실행됩니다."),
    }
    // 출력(표준 에러): (빈 줄)
    // 출력(표준 에러): thread '<unnamed>' (스레드 ID) panicked at part0-getting-started/ch15-concurrency/examples/02_move_and_return.rs:76:14:
    // 출력(표준 에러): index out of bounds: the len is 0 but the index is 0
    // 출력(표준 에러): note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    // 출력:     스레드가 패닉했습니다. main은 계속 실행됩니다.
    // '<unnamed>'는 이름을 붙이지 않은 스레드라는 뜻이고, 괄호 안의 스레드 ID는 실행마다 다릅니다.
}
