// 기대하는 결과: 2024 에디션에서 실패, 2021 에디션에서는 경고와 함께 통과
// 핵심 메시지: creating a shared reference to mutable static (에러 코드 없음, 린트 static_mut_refs)
//
// `static_mut_refs`는 2021 에디션까지 warn, 2024 에디션에서 deny인 린트입니다. `cargo fix --edition`은
// 이 린트를 경고로 보여 줄 뿐 자동 수정이 없습니다. 값을 대입하는 줄은 참조를 만들지 않으므로
// 문제가 없고, `.clone()`이 `&LAST_ERROR`를 몰래 만드는 줄이 걸립니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/static_mut_shared_reference.rs --out-dir /tmp/rust-p2-ch6   → 에러
//   rustc --edition 2021 compile_fail/static_mut_shared_reference.rs --out-dir /tmp/rust-p2-ch6   → 경고와 함께 통과

static mut LAST_ERROR: Option<String> = None;

fn record(message: &str) {
    // SAFETY: 이 프로그램은 스레드를 만들지 않으므로 LAST_ERROR에 동시에 접근하는 코드가 없습니다.
    unsafe {
        LAST_ERROR = Some(message.to_string()); // 대입: 참조를 만들지 않음
    }
}

fn last_error() -> Option<String> {
    // SAFETY: record와 같은 이유입니다.
    unsafe { LAST_ERROR.clone() } // → 2024 에디션에서 에러(공유 참조를 만듦)
}

fn main() {
    record("금액이 없습니다");
    println!("{:?}", last_error());
}
