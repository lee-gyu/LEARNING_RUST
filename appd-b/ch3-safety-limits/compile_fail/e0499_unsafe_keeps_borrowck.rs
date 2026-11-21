// 기대하는 에러: E0499
// 핵심 메시지: cannot borrow `scores` as mutable more than once at a time
// 함께 나오는 경고: unnecessary `unsafe` block
//
// `unsafe` 블록은 대여 검사기를 끄지 않습니다. `unsafe`가 허용하는 것은 원시 포인터 역참조,
// unsafe 함수 호출 같은 다섯 가지 일뿐이고(Rustonomicon), 같은 값을 동시에 두 번 가변으로
// 빌리는 코드는 블록 밖에서와 똑같이 거부됩니다. 블록 안에서 그 다섯 가지를 하나도 하지 않으므로
// 컴파일러는 이 블록이 필요 없다는 경고도 냅니다.
//
// 고치는 법: `unsafe`를 지우고, 겹치지 않는 두 부분이 필요하면 `split_at_mut`처럼 그 일을 해 주는
// 안전한 API를 씁니다(예제 05).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch3-safety-limits compile_fail/e0499_unsafe_keeps_borrowck.rs

fn main() {
    let mut scores = vec![10, 20, 30];
    unsafe {
        let first = &mut scores[0];
        let last = &mut scores[2]; // E0499
        *first += *last;
    }
    println!("{scores:?}");
}
