// 스레드를 만들고(spawn) 끝나기를 기다리는(join) 방법을 보여 줍니다.
// - join을 어디에 두느냐에 따라 두 스레드가 동시에 도는지, 차례로 도는지가 달라집니다.
// - main이 끝나면 아직 일하던 스레드도 함께 끝납니다.
//
// 실행: cargo run -p p0-ch15-concurrency --example 01_spawn_and_join
//
// 주의: 여러 스레드가 번갈아 출력하는 줄의 순서는 운영체제가 스레드를 언제 실행하느냐에
// 달려 있어서 실행할 때마다 달라질 수 있습니다. 아래 `// 출력:` 주석에는 순서가 정해진
// 줄만 적고, 순서가 달라지는 부분은 그렇다고 적었습니다.

use std::thread;
use std::time::Duration;

fn main() {
    println!("1. join을 뒤에 두면 두 스레드가 동시에 실행됩니다");
    // thread::spawn은 클로저를 받아 새 스레드에서 실행하고, 곧바로 JoinHandle을 돌려줍니다.
    // 새 스레드가 일을 마칠 때까지 기다리지 않으므로 main은 바로 다음 줄로 넘어갑니다.
    let handle = thread::spawn(|| {
        for i in 1..=4 {
            println!("    [새 스레드] {i}");
            // 잠깐 멈추면 운영체제가 그 사이에 다른 스레드를 실행할 기회가 생깁니다.
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..=3 {
        println!("    [main] {i}");
        thread::sleep(Duration::from_millis(1));
    }

    // join은 그 스레드가 끝날 때까지 지금 스레드(main)를 멈춰 세웁니다(블록).
    // 여기서 기다리므로 [새 스레드] 4까지 반드시 출력된 뒤에 다음 줄로 넘어갑니다.
    handle.join().unwrap();
    println!("    → 두 스레드가 모두 끝났습니다");
    // 출력: [새 스레드] 1~4와 [main] 1~3이 섞여서 나옵니다. 섞이는 순서는 실행마다 다를 수 있습니다.
    // 출력:     → 두 스레드가 모두 끝났습니다   (항상 마지막)

    println!("2. join을 바로 부르면 새 스레드가 끝난 뒤에야 main이 진행합니다");
    let handle = thread::spawn(|| {
        for i in 1..=4 {
            println!("    [새 스레드] {i}");
            thread::sleep(Duration::from_millis(1));
        }
    });

    // 반복문보다 먼저 join을 부르면 main은 여기서 새 스레드를 끝까지 기다립니다.
    // 스레드를 만들었지만 결과적으로 "동시에" 일한 구간은 없습니다.
    handle.join().unwrap();

    for i in 1..=3 {
        println!("    [main] {i}");
        thread::sleep(Duration::from_millis(1));
    }
    // 출력:     [새 스레드] 1
    // 출력:     [새 스레드] 2
    // 출력:     [새 스레드] 3
    // 출력:     [새 스레드] 4
    // 출력:     [main] 1
    // 출력:     [main] 2
    // 출력:     [main] 3

    println!("3. 스레드를 여러 개 만들고 JoinHandle을 모아 두었다가 한꺼번에 기다립니다");
    let mut handles = Vec::new();
    for id in 1..=3 {
        // id는 i32(Copy 타입)라서 move로 복사본이 각 스레드로 들어갑니다.
        // move가 왜 필요한지는 02_move_and_return.rs에서 자세히 봅니다.
        handles.push(thread::spawn(move || {
            println!("    [스레드 {id}] 일하는 중");
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("    → 세 스레드가 모두 끝났습니다");
    // 출력: [스레드 1]~[스레드 3]의 세 줄이 나오며, 순서는 실행마다 다를 수 있습니다.
    // 출력:     → 세 스레드가 모두 끝났습니다   (항상 세 줄 뒤)

    println!("4. join하지 않으면 main이 끝날 때 남은 스레드도 함께 끝나 버립니다");
    // 이 스레드는 다 하려면 5 x 50ms = 250ms쯤 걸립니다.
    thread::spawn(|| {
        for i in 1..=5 {
            println!("    [새 스레드] {i}/5");
            thread::sleep(Duration::from_millis(50));
        }
        println!("    [새 스레드] 끝까지 마쳤습니다"); // 이 줄은 출력되지 않습니다.
    });

    // main은 120ms만 기다리고 끝납니다. JoinHandle을 버렸으므로 기다릴 방법도 없습니다.
    thread::sleep(Duration::from_millis(120));
    println!("    [main] 끝납니다 (join하지 않음)");
    // 출력: [새 스레드] 1/5부터 몇 줄이 나오다가 중간에 끊깁니다. 이 환경에서는 보통 3/5까지였지만
    //       몇 번까지 찍히는지는 실행마다 다를 수 있습니다. "끝까지 마쳤습니다"는 나오지 않습니다.
    // 출력:     [main] 끝납니다 (join하지 않음)
}
