//! 2024 에디션 바이너리 크레이트입니다.
//!
//! 2015 에디션 크레이트(jobs_2015)와 2021 에디션 크레이트(idgen_2021)를 가져와 한 프로그램으로
//! 링크합니다. 세 크레이트는 같은 rustc가 각자의 에디션으로 컴파일합니다.
//!
//! 실행(이 프로젝트 디렉터리에서): cargo run
//! 크레이트마다 넘어가는 에디션 보기: cargo clean && cargo build -v

use std::any::type_name_of_val;

// jobs_2015::first_item!, idgen_2021::first_item!과 본문이 똑같은 매크로를 이 크레이트(2024)에도
// 정의해 둡니다. 3번 구역에서 세 매크로의 결과 타입을 비교합니다.
macro_rules! first_item {
    ($array:expr) => {
        $array.into_iter().next()
    };
}

fn main() {
    println!("1. 에디션이 다른 크레이트의 함수 부르기");
    // async와 await는 2018 에디션부터 키워드라서 이 크레이트에서 `jobs_2015::async(..)`라고
    // 쓰면 컴파일 에러입니다. 앞에 r#를 붙인 raw 식별자는 "키워드가 아니라 이름"이라는 표시입니다.
    let mut job = jobs_2015::r#async("백업", 3);
    // 2015 크레이트가 정의한 타입과 메서드도 그대로 씁니다.
    job.toggle_pause();
    let report = jobs_2015::r#await(job);
    println!("    r#async → r#await: {report}"); // 출력: r#async → r#await: 백업: 3단계 일시정지

    // gen은 2024 에디션의 예약 키워드입니다. 2021 크레이트의 gen 함수도 r#gen으로 부릅니다.
    let small = idgen_2021::r#gen("user", 7);
    let too_big = idgen_2021::r#gen("user", 70_000);
    println!("    r#gen: {small:?}, {too_big:?}"); // 출력: r#gen: Some("user-00007"), None

    println!("2. 크레이트마다 자기 에디션의 문법을 씁니다");
    // let 체인은 2024 에디션에서만 쓸 수 있는 문법입니다. 이 크레이트가 2024라서 쓸 수 있고,
    // 체인 안에서 부르는 함수를 만든 크레이트의 에디션(2015, 2021)과는 상관없습니다.
    if let Some(id) = idgen_2021::r#gen("job", 42)
        && jobs_2015::initial(&id) == 'j'
    {
        println!("    let 체인 안에서 두 크레이트의 함수를 함께: {id}"); // 출력: let 체인 안에서 두 크레이트의 함수를 함께: job-00042
    }

    println!("3. 매크로 본문은 매크로를 정의한 크레이트의 에디션으로 읽힙니다");
    // run_now!의 본문은 async를 변수 이름으로 씁니다. 그 토큰은 jobs_2015(2015)에서 왔으므로
    // 이 크레이트(2024) 안에서 펼쳐져도 식별자로 읽힙니다.
    let done = jobs_2015::run_now!("정리");
    println!("    jobs_2015::run_now!: {done}"); // 출력: jobs_2015::run_now!: 정리: 1단계 완료

    // 본문이 똑같은 세 매크로에 같은 배열을 넘깁니다. 결과 타입은 매크로를 정의한 크레이트의
    // 에디션이 정합니다. 2015 에디션의 규칙에서 배열의 .into_iter()는 참조를 내놓습니다.
    let numbers = [10, 20, 30];
    let a = jobs_2015::first_item!(numbers);
    let b = idgen_2021::first_item!(numbers);
    let c = first_item!(numbers);
    let (ta, tb, tc) = (
        type_name_of_val(&a),
        type_name_of_val(&b),
        type_name_of_val(&c),
    );
    println!("    2015 크레이트의 매크로: {a:?}, {ta}"); // 출력: 2015 크레이트의 매크로: Some(10), core::option::Option<&i32>
    println!("    2021 크레이트의 매크로: {b:?}, {tb}"); // 출력: 2021 크레이트의 매크로: Some(10), core::option::Option<i32>
    println!("    2024 크레이트의 매크로: {c:?}, {tc}"); // 출력: 2024 크레이트의 매크로: Some(10), core::option::Option<i32>
}
