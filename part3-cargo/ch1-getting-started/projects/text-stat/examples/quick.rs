// `examples/`의 파일은 라이브러리를 쓰는 방법을 보여 주는 작은 프로그램입니다.
// 라이브러리 사용자와 똑같이 공개 API만 쓸 수 있습니다.
//
// 실행: cargo run --example quick

fn main() {
    let counts = text_stat::count("one two\nthree");
    println!("{counts:?}");
}
