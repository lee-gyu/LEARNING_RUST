// 기대하는 에러: symbol `answer` is already defined (에러 코드 없음)
//
// 보통 함수의 심볼 이름에는 크레이트와 모듈 경로가 들어가므로(이름 맹글링) 다른 모듈의 같은 이름이 부딪치지 않습니다.
// #[unsafe(no_mangle)]은 그 보호를 끄고 함수 이름을 그대로 심볼 이름으로 내보냅니다. 같은 크레이트 안에서 두 모듈이
// 같은 이름을 내보내면 컴파일러가 이렇게 거부합니다.
// 크레이트 사이라면 컴파일러가 서로를 보지 못하므로, macOS에서는 링커 경고(ld: duplicate symbol)만 나고 빌드가
// 성공하며 한쪽 정의가 쓰입니다. 2024 에디션이 no_mangle을 unsafe(...)로 감싸게 한 이유가 이런 충돌을
// 컴파일러가 모두 막지 못하기 때문입니다(Part 2 ch4 2.8절).
// 고치는 방법: 내보낼 이름을 겹치지 않게 짓습니다(라이브러리 접두사 등). C에서 부를 필요가 없으면 no_mangle을 뺍니다.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch13-headers-build-templates compile_fail/no_mangle_symbol_clash.rs

mod sensor {
    #[unsafe(no_mangle)]
    pub extern "C" fn answer() -> i32 {
        1
    }
}

mod display {
    #[unsafe(no_mangle)]
    pub extern "C" fn answer() -> i32 {
        2
    }
}

fn main() {
    println!("{} {}", sensor::answer(), display::answer());
}
