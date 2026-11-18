// 스레드 다루기: thread::Builder로 이름과 스택 크기를 정하고 생성 실패를 io::Result로 받기,
// 현재 스레드 정보(이름, ThreadId), available_parallelism, thread_local!,
// 그리고 이름 붙인 스레드가 패닉했을 때 패닉 훅과 join이 보는 것.
//
// 실행: cargo run -p p1-ch11-threads-sync --example 02_thread_builder
//
// 4번 구역의 값은 실행하는 컴퓨터마다 다릅니다.
// 6번 구역은 일부러 스레드 하나를 패닉시킵니다. 바꿔 둔 패닉 훅이 표준 에러에 한 줄을
// 남기고, 패닉은 join으로 받으므로 종료 코드는 0입니다.

use std::cell::Cell;
use std::io;
use std::panic;
use std::thread;

thread_local! {
    // 스레드마다 따로 가지는 전역 변수입니다. 초기값이 상수이면 const { }로 감싸는 것이
    // 권장됩니다(clippy::missing_const_for_thread_local).
    static CALLS: Cell<u32> = const { Cell::new(0) };
}

fn record_call() -> u32 {
    // LocalKey<Cell<T>>에는 get/set이 바로 있습니다(1.73.0).
    CALLS.set(CALLS.get() + 1);
    CALLS.get()
}

fn main() -> io::Result<()> {
    println!("1. Builder로 이름을 붙이면 스레드 안에서 thread::current().name()으로 읽습니다");
    let named = thread::Builder::new()
        .name("작업자-1".to_string())
        .stack_size(256 * 1024) // 256 KiB. 기본값은 플랫폼마다 다르며 현재 2 MiB입니다.
        .spawn(|| thread::current().name().map(String::from))?; // io::Result라서 ?를 씁니다.
    let unnamed = thread::spawn(|| thread::current().name().map(String::from));
    println!("    Builder로 만든 스레드: {:?}", named.join().unwrap());
    println!("    thread::spawn 스레드: {:?}", unnamed.join().unwrap());
    println!("    main 스레드:          {:?}", thread::current().name());
    // 출력: Builder로 만든 스레드: Some("작업자-1")
    // 출력: thread::spawn 스레드: None
    // 출력: main 스레드:          Some("main")

    // 스코프 스레드에도 같은 설정을 쓸 수 있습니다: spawn_scoped(스코프, 클로저)
    let label = "스코프";
    let scoped_name = thread::scope(|s| {
        thread::Builder::new()
            .name(format!("{label}-작업자"))
            .spawn_scoped(s, || thread::current().name().map(String::from))
            .map(|h| h.join().unwrap())
    })?;
    println!("    spawn_scoped 스레드:  {scoped_name:?}");
    // 출력: spawn_scoped 스레드:  Some("스코프-작업자")

    println!("2. Builder::spawn은 실패를 패닉 대신 io::Result의 Err로 알려 줍니다");
    // usize::MAX 바이트짜리 스택은 만들 수 없습니다. thread::spawn이었다면 여기서 패닉합니다.
    match thread::Builder::new().stack_size(usize::MAX).spawn(|| ()) {
        Ok(handle) => {
            handle.join().unwrap();
            println!("    스레드를 만들었습니다");
        }
        Err(e) => println!("    스레드를 만들지 못했습니다: {e} (종류: {:?})", e.kind()),
    }
    // 출력: 스레드를 만들지 못했습니다: invalid stack size (종류: InvalidInput)  ← macOS 기준

    println!("3. ThreadId는 스레드를 구별하는 값입니다");
    let handle = thread::spawn(|| thread::current().id());
    let id_from_handle = handle.thread().id(); // 핸들에서도 같은 스레드 정보를 얻습니다.
    let id_inside = handle.join().unwrap();
    println!(
        "    핸들의 id == 스레드 안의 id: {}",
        id_from_handle == id_inside
    );
    println!(
        "    main의 id == 작업 스레드의 id: {}",
        thread::current().id() == id_inside
    );
    // ThreadId의 디버그 형식(예: ThreadId(2))은 실행 환경에 따라 달라질 수 있어 비교만 합니다.
    // 출력: 핸들의 id == 스레드 안의 id: true
    // 출력: main의 id == 작업 스레드의 id: false

    println!("4. available_parallelism은 동시에 돌릴 스레드 수의 추정값입니다");
    // Result<NonZero<usize>>를 돌려줍니다. 알 수 없는 환경에서는 Err이므로 기본값을 정해 둡니다.
    let workers = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!("    이 컴퓨터의 추정값: {workers}");
    // 출력: 이 컴퓨터의 추정값: (실행 환경마다 다름)

    println!("5. thread_local! 변수는 스레드마다 따로 있습니다");
    record_call();
    record_call();
    let in_main = record_call();
    let in_worker = thread::spawn(record_call).join().unwrap();
    println!("    main에서 세 번 부른 뒤: {in_main}, 새 스레드에서 한 번 부른 뒤: {in_worker}");
    println!("    main의 값은 그대로: {}", CALLS.get());
    // 출력: main에서 세 번 부른 뒤: 3, 새 스레드에서 한 번 부른 뒤: 1
    // 출력: main의 값은 그대로: 3

    println!("6. 패닉 훅은 프로세스 전체에 하나라서 어느 스레드의 패닉에도 불립니다");
    // 훅은 패닉한 스레드에서 실행되므로, 그 안에서 thread::current()는 패닉한 스레드입니다.
    panic::set_hook(Box::new(|info| {
        let current = thread::current();
        let name = current.name().unwrap_or("이름 없음");
        let message = info.payload_as_str().unwrap_or("문자열이 아닌 페이로드");
        eprintln!("    [훅] 스레드 '{name}'의 패닉: {message}");
    }));
    let result = thread::Builder::new()
        .name("계산기".to_string())
        .spawn(|| {
            let divisor = "0".parse::<i32>().unwrap();
            100 / divisor
        })?
        .join();
    // join의 Err에는 패닉 페이로드(Box<dyn Any + Send>)가 들어 있습니다.
    match result {
        Ok(v) => println!("    결과: {v}"),
        Err(payload) => {
            let text = payload.downcast_ref::<&str>().copied().unwrap_or("?");
            println!("    join이 받은 Err의 페이로드: {text}");
        }
    }
    let _ = panic::take_hook(); // 기본 훅으로 되돌립니다.
    // 출력(표준 에러): [훅] 스레드 '계산기'의 패닉: attempt to divide by zero
    // 출력: join이 받은 Err의 페이로드: attempt to divide by zero

    Ok(())
}
