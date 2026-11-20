// 기대하는 에러: E0690
//   transparent struct needs at most one non-trivial field, but has 2
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0690_transparent_two_fields.rs
//
// #[repr(transparent)]는 "이 타입은 안쪽 필드 하나와 같은 배치, 같은 호출 규약"이라는 약속입니다.
// 크기가 0이 아닌 필드가 둘이면 어느 쪽과 같아야 하는지 정할 수 없으므로 거부됩니다.
// (크기가 0이고 메모리 정렬이 1인 PhantomData 같은 필드는 몇 개를 더해도 됩니다.)
// 해결: 필드가 여럿인 타입을 C와 주고받으려면 #[repr(C)]를 붙이고, C 쪽에서도 구조체로 받습니다.
//   뉴타입(필드 하나)일 때만 #[repr(transparent)]를 씁니다.

#[repr(transparent)]
pub struct Range(pub f64, pub f64);

extern "C" fn width(r: Range) -> f64 {
    r.1 - r.0
}

fn main() {
    println!("{}", width(Range(1.5, 4.0)));
}
