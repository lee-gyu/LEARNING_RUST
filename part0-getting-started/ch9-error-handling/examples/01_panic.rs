// panic!이 일어나면 무슨 일이 생기는지 관찰합니다.
// - 패닉 메시지: 어느 파일의 몇 번째 줄, 몇 번째 열에서 무슨 이유로 멈췄는지 알려 줍니다.
// - 되감기(unwinding): 패닉한 뒤에도 스택에 있던 값들이 안쪽 함수부터 차례로 drop됩니다.
// - 패닉으로 끝난 프로그램의 종료 코드는 101입니다.
//
// 주의: 이 예제는 2번 구역에서 **일부러 패닉하며 끝납니다**(종료 코드 101).
//
// 실행: cargo run -p p0-ch9-error-handling --example 01_panic
// 종료 코드 확인: 위 명령 바로 뒤에 echo $?
// 백트레이스 보기: RUST_BACKTRACE=1 cargo run -p p0-ch9-error-handling --example 01_panic

// 버려질 때 메시지를 출력하는 관찰용 타입입니다(ch4, ch8에서 쓴 것과 같습니다).
// `Drop` 트레이트는 ch14에서 배웁니다. 지금은 "버려질 때 `[drop] 이름`을 출력하는 값"으로 보면 됩니다.
struct Noisy {
    name: String,
}

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.name);
    }
}

fn noisy(name: &str) -> Noisy {
    Noisy {
        name: String::from(name),
    }
}

// 창고 재고에서 amount개를 출고하고 남은 재고를 돌려줍니다.
// 이 프로그램에서는 "재고보다 많이 출고하는 일"이 일어나면 안 되는 버그라고 가정합니다.
// 버그를 만나면 엉뚱한 값으로 계속 진행하기보다 그 자리에서 멈추는 편이 안전하므로 panic!을 씁니다.
fn ship(stock: u32, amount: u32) -> u32 {
    let _slip = noisy("ship의 출고 전표");

    if amount > stock {
        // panic!은 println!과 같은 서식 문자열을 받습니다.
        // 이 검사가 없어도 아래 뺄셈이 u32 범위를 벗어나 패닉하지만(ch3의 오버플로),
        // 직접 쓴 메시지가 무엇이 잘못됐는지 훨씬 분명하게 알려 줍니다.
        panic!("출고 수량({amount})이 재고({stock})보다 많습니다");
    }

    stock - amount
} // 정상적으로 끝나면 여기서 _slip이 drop됩니다.

fn main() {
    let _ledger = noisy("main의 장부");

    println!("1. 조건을 지키는 동안은 평소처럼 실행됩니다");
    let stock = ship(10, 3);
    println!("    남은 재고: {stock}");
    // 출력:
    //     [drop] ship의 출고 전표
    //     남은 재고: 7

    println!("2. 조건을 어기면 panic!이 프로그램을 멈춥니다");
    let stock = ship(stock, 20);
    println!("    이 줄은 실행되지 않습니다: {stock}");
    // 출력 (패닉 메시지는 표준 에러로 나갑니다):
    //
    //     thread 'main' (스레드 ID) panicked at part0-getting-started/ch9-error-handling/examples/01_panic.rs:40:9:
    //     출고 수량(20)이 재고(7)보다 많습니다
    //     note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    //         [drop] ship의 출고 전표
    //         [drop] main의 장부
    //
    // 스레드 ID는 실행할 때마다 달라집니다.
    // 패닉 메시지 뒤에 [drop] 두 줄이 찍히는 것이 되감기입니다. 패닉이 함수를 빠져나가면서
    // 안쪽 함수(ship)의 값부터 바깥 함수(main)의 값까지 차례로 정리합니다.
}
