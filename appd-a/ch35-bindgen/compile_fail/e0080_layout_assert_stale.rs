// 기대하는 에러: E0080 index out of bounds: the length is 1 but the index is 8
//               (max의 오프셋이 24가 아니라 32라서 `32 - 24`번 원소를 읽으려 함)
//
// 확인: rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0080_layout_assert_stale.rs
//
// 아래 구조체는 v1 헤더에 맞춰 쓴 정의(min이 max 앞)이고, 레이아웃 검사는 bindgen 0.73.2가
// v2 헤더(max가 min 앞)에서 생성한 것입니다. 숫자를 바뀐 헤더에서 다시 재면 낡은 정의는
// 컴파일 때 걸립니다. 반대로 정의와 검사를 함께 커밋해 두고 둘 다 다시 생성하지 않으면,
// 둘이 함께 낡아 아무것도 잡지 못합니다(examples/02_const_layout_asserts.rs).

// v1 헤더를 보고 쓴 정의(낡음)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tally_summary {
    pub count: u64,
    pub below: u64,
    pub above: u64,
    pub min: i64,
    pub max: i64,
}

// v2 헤더에서 생성한 레이아웃 검사
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of tally_summary"][::std::mem::size_of::<tally_summary>() - 40usize];
    ["Alignment of tally_summary"][::std::mem::align_of::<tally_summary>() - 8usize];
    ["Offset of field: tally_summary::count"]
        [::std::mem::offset_of!(tally_summary, count) - 0usize];
    ["Offset of field: tally_summary::below"]
        [::std::mem::offset_of!(tally_summary, below) - 8usize];
    ["Offset of field: tally_summary::above"]
        [::std::mem::offset_of!(tally_summary, above) - 16usize];
    ["Offset of field: tally_summary::max"][::std::mem::offset_of!(tally_summary, max) - 24usize];
    ["Offset of field: tally_summary::min"][::std::mem::offset_of!(tally_summary, min) - 32usize];
};

fn main() {
    let s = tally_summary { count: 1, below: 0, above: 0, min: -3, max: 250 };
    println!("{s:?}");
}
