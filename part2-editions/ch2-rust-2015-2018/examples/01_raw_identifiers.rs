// raw 식별자(raw identifier) `r#이름`: 키워드와 같은 이름을 식별자로 쓰는 문법
//
// 실행: cargo run -p p2-ch2-rust-2015-2018 --example 01_raw_identifiers
//
// 2018 에디션에서 async, await, dyn이 키워드가 되고 try가 예약 키워드가 되었습니다.
// 그 전(2015 에디션)에 이 단어를 이름으로 쓴 크레이트가 있으므로, 새 에디션의 코드에서
// 그 이름을 부를 방법이 필요합니다. r#를 붙이면 "이것은 키워드가 아니라 이름이다"라는 뜻이 됩니다.
// 이 예제는 2024 에디션으로 컴파일됩니다.

/// 2015 에디션 크레이트가 공개한 API를 흉내 낸 모듈입니다.
/// (진짜 2015 크레이트와 함께 쓰는 예는 projects/scoreboard에 있습니다.)
mod legacy {
    /// 2015 크레이트의 소스에는 `pub fn try(text: &str) -> ...`로 적혀 있었을 함수입니다.
    /// 2018 이후 에디션에서 같은 이름을 정의하려면 r#try라고 적어야 합니다.
    pub fn r#try(text: &str) -> Option<u32> {
        text.trim().parse().ok()
    }

    /// 필드 이름이 키워드인 구조체입니다. 설정 파일이나 JSON의 "type", "async" 같은 항목 이름을
    /// 필드 이름에 그대로 옮기고 싶을 때 이런 모양이 나옵니다.
    #[derive(Debug)]
    pub struct Job {
        pub r#type: &'static str,
        pub r#async: bool,
    }

    /// 모듈 이름에도 쓸 수 있습니다.
    pub mod r#dyn {
        pub struct Marker;

        pub fn location() -> &'static str {
            module_path!()
        }
    }
}

fn main() {
    println!("1. 키워드와 같은 이름을 r#로 부릅니다");
    println!("    legacy::r#try(\"42\")  = {:?}", legacy::r#try("42")); // 출력:     legacy::r#try("42")  = Some(42)
    println!("    legacy::r#try(\"4X\")  = {:?}", legacy::r#try("4X")); // 출력:     legacy::r#try("4X")  = None
    let job = legacy::Job {
        r#type: "batch",
        r#async: true,
    };
    // 필드에 접근할 때도 r#를 붙입니다. job.type이라고 쓰면 type 키워드로 읽혀 파싱 에러가 납니다.
    println!(
        "    job.r#type = {}, job.r#async = {}",
        job.r#type, job.r#async
    ); // 출력:     job.r#type = batch, job.r#async = true

    println!("2. r#는 이름의 일부가 아닙니다");
    // r#count와 count는 같은 식별자입니다. r#는 "키워드로 읽지 말라"는 표시일 뿐이라서,
    // 키워드가 아닌 이름에 붙이면 아무 효과가 없습니다.
    let r#count = 3;
    println!("    r#count로 만들고 count로 읽기: {}", count + 1); // 출력:     r#count로 만들고 count로 읽기: 4
    // gen은 2024 에디션에서 예약 키워드가 되었으므로 이 예제(2024)에서는 r#가 필요합니다.
    // 어떤 단어에 r#가 필요한지는 그 코드를 컴파일하는 크레이트의 에디션이 정합니다.
    let r#gen = "2024 에디션에서는 r#gen";
    println!("    {}", r#gen); // 출력:     2024 에디션에서는 r#gen

    println!("3. 출력과 매크로에서 보이는 모습");
    // derive(Debug)는 r#를 뺀 이름을 출력합니다. 필드의 "진짜 이름"은 type이기 때문입니다.
    println!("    {:?}", job); // 출력:     Job { type: "batch", async: true }
    // 서식 문자열의 이름 자리에는 키워드를 r# 없이 씁니다. {r#type}이라고 쓰면 컴파일 에러입니다
    // (invalid format string: raw identifiers are not supported).
    let r#type = job.r#type;
    println!("    서식 문자열의 {{type}}: {type}"); // 출력:     서식 문자열의 {type}: batch
    // stringify!는 받은 토큰을 그대로 문자열로 만들므로 r#가 남습니다.
    println!("    stringify!: {}", stringify!(legacy::r#try(x))); // 출력:     stringify!: legacy::r#try(x)
    // module_path!도 r#를 남깁니다. 반대로 type_name은 r#를 뺀 이름을 돌려줍니다.
    println!("    module_path!: {}", legacy::r#dyn::location()); // 출력:     module_path!: 01_raw_identifiers::legacy::r#dyn
    println!(
        "    type_name: {}",
        std::any::type_name::<legacy::r#dyn::Marker>()
    ); // 출력:     type_name: 01_raw_identifiers::legacy::dyn::Marker
}
