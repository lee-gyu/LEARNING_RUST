// 기대하는 에러: E0277 — the size for values of type `str` cannot be known at compilation time
// 확인(챕터 디렉터리에서): rustc --edition 2024 compile_fail/e0277_unsized_local.rs
//
// str은 길이가 값마다 다른 동적 크기 타입(DST)입니다. 지역 변수는 스택에 자리를 미리 잡아야 하므로
// 크기를 컴파일 시점에 알 수 있어야(Sized) 합니다. *greeting은 참조를 벗긴 str 값 자체라서 담을 수 없습니다.
// 해결: 참조(&str)나 소유하는 타입(String, Box<str>)으로 담습니다. 동적 크기 타입은 항상 포인터 뒤에 둡니다.

fn main() {
    let greeting: &str = "안녕하세요";
    let owned: str = *greeting;
    println!("{}", owned.len());
}
