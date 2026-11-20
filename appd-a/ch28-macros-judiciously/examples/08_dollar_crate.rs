// 지침 6: 내보내는 매크로는 자기 크레이트의 항목을 `$crate::경로`로 가리킵니다.
//
// macro_rules! 매크로의 위생은 지역 변수와 레이블에만 적용됩니다. 함수, 타입, 모듈 같은 항목의 이름은
// 매크로를 "정의한 곳"이 아니라 "펼친 곳(부른 곳)"에서 찾습니다. 그래서 매크로 본문에
// `record(..)`라고만 쓰면 다른 모듈에서 불렀을 때 찾지 못하고(compile_fail/e0425_item_path_at_call_site.rs),
// `crate::metrics::record(..)`라고 쓰면 다른 크레이트에서 불렀을 때 "그 크레이트"의 루트를 가리킵니다.
// `$crate`는 어디서 펼쳐지든 매크로를 정의한 크레이트를 가리키는 특별한 메타변수입니다.
//
// 실행: cargo run -p er-ch28-macros-judiciously --example 08_dollar_crate

mod metrics {
    use std::sync::atomic::{AtomicU32, Ordering};

    static EVENTS: AtomicU32 = AtomicU32::new(0);

    pub fn record(event: &str) -> u32 {
        let n = EVENTS.fetch_add(1, Ordering::Relaxed) + 1;
        println!("  [{n}] {event}");
        n
    }

    // 비권장: `crate::`는 매크로를 "부른" 크레이트의 루트입니다. 이 크레이트 안에서만 우연히 맞습니다.
    // Clippy가 기본으로 경고합니다(crate_in_macro_def). 다른 크레이트에서 깨지는 모습은
    // projects/tabular/user_fails에서 봅니다.
    #[macro_export]
    #[expect(
        clippy::crate_in_macro_def,
        reason = "비교용 비권장 판: crate::는 부른 크레이트를 가리킨다"
    )]
    macro_rules! track_crate {
        ($event:expr) => {
            crate::metrics::record($event)
        };
    }

    // 권장: `$crate::`는 어디서 펼쳐지든 이 크레이트입니다. 모듈 경로까지 다 적어야 합니다.
    #[macro_export]
    macro_rules! track {
        ($event:expr) => {
            $crate::metrics::record($event)
        };
    }
}

mod handlers {
    // #[macro_export] 매크로는 모듈 안에서 정의해도 크레이트 루트에 놓입니다.
    // 그래서 `crate::metrics::track!`이 아니라 `crate::track!`으로 부릅니다(앞의 것은 E0433).
    pub fn login() {
        crate::track!("handlers::login에서 track!");
        crate::track_crate!("handlers::login에서 track_crate!");
    }

    // 프렐류드의 이름도 부르는 쪽에서 가려질 수 있습니다. 이 모듈은 `Result`를 인자 하나짜리 별칭으로 바꿨습니다.
    // 본문에 `Result<$ty, String>`이라고 쓴 매크로를 여기서 펼치면 E0107(제네릭 인자 개수가 다름)입니다.
    #[derive(Debug)]
    pub struct HandlerError(pub String);
    pub type Result<T> = core::result::Result<T, HandlerError>;

    pub fn port(text: &str) -> Result<u16> {
        crate::parse_as!(text, u16).map_err(HandlerError)
    }
}

// 문자열을 `$ty`로 읽고, 실패하면 "무엇을 무엇으로 읽다 실패했는지"를 식의 글자 그대로 담은 에러를 돌려줍니다.
// 식의 글자가 필요하므로 매크로입니다(예제 03). 본문의 경로는 모두 `::`로 시작해 부르는 쪽의 별칭이나
// 같은 이름의 항목에 가려지지 않게 했습니다.
#[macro_export]
macro_rules! parse_as {
    ($text:expr, $ty:ty) => {{
        let parsed: ::core::result::Result<$ty, ::std::string::String> =
            $text.trim().parse::<$ty>().map_err(|e| {
                ::std::format!(
                    "`{}`을(를) {}(으)로 읽지 못함: {e}",
                    ::core::stringify!($text),
                    ::core::stringify!($ty)
                )
            });
        parsed
    }};
}

fn main() {
    println!("1. 이 크레이트 안에서는 crate::도 $crate::도 같은 곳을 가리킴");
    track!("main에서 track!"); // 출력:   [1] main에서 track!
    track_crate!("main에서 track_crate!"); // 출력:   [2] main에서 track_crate!
    handlers::login();
    // 출력:   [3] handlers::login에서 track!
    // 출력:   [4] handlers::login에서 track_crate!

    println!("2. 부르는 쪽의 Result 별칭에 가려지지 않는 경로");
    let ok = handlers::port(" 8080 ");
    let bad = handlers::port("80a").unwrap_err().0;
    println!("  port(\" 8080 \") = {ok:?}"); // 출력:   port(" 8080 ") = Ok(8080)
    println!("  port(\"80a\")    → {bad}"); // 출력:   port("80a")    → `text`을(를) u16(으)로 읽지 못함: invalid digit found in string
}
