// 기대하는 에러: E0277 — the size for values of type `dyn std::error::Error` cannot be known at compilation time
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_box_dyn_error_is_not_error.rs --out-dir /tmp/rust-p1-ch8
//
// report는 E: Error인 타입을 받는 제네릭 함수인데, Box<dyn Error>를 넘겼습니다.
// 표준 라이브러리의 `impl<E: Error> Error for Box<E>`는 E에 크기가 정해진 타입(Sized)을 요구하므로
// Box<dyn Error> 자신은 Error를 구현하지 않습니다. 구현했다면 `impl<E: Error> From<E> for Box<dyn Error>`와
// `impl<T> From<T> for T`가 Box<dyn Error>에서 겹치기 때문입니다.
// 해결: 함수가 트레이트 객체를 받게 하고(fn report(err: &dyn Error)) report(e.as_ref())로 Box 안의 에러를 빌려 넘깁니다.
//       E: Error 바운드에 ?Sized를 더해(fn report<E: Error + ?Sized>(err: &E)) report(&*e)로 넘겨도 됩니다.

use std::error::Error;

fn report<E: Error>(err: &E) {
    println!("에러: {err}");
    let mut current = err.source();
    while let Some(cause) = current {
        println!("  원인: {cause}");
        current = cause.source();
    }
}

fn parse(text: &str) -> Result<i32, Box<dyn Error>> {
    Ok(text.parse::<i32>()?)
}

fn main() {
    if let Err(e) = parse("12a") {
        report(&e);
    }
}
