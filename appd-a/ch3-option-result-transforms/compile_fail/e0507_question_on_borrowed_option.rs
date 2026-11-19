// 기대하는 에러: error[E0507]: cannot move out of `self.tracking_no` which is behind a shared reference
// 핵심 메시지: `self.tracking_no` moved due to the question mark operator
//             note: the question mark operator is desugared into a call to `std::ops::Try::branch`,
//                   which takes ownership of the receiver `self`, which moves `self.tracking_no`
//             help: consider cloning the value if the performance cost is acceptable
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0507_question_on_borrowed_option.rs --out-dir /tmp/rust-er-ch3
//
// match &self.tracking_no { Some(no) => .., None => return None }를 ?로 바꾸면서 &도 함께 지워 버린 모양입니다.
// match는 참조를 조건식으로 받을 수 있지만, ?는 Option을 통째로 가져갑니다(Try::branch가 self를 받음).
// &self로 빌린 구조체의 필드는 가져갈 수 없으므로 E0507입니다.
// 컴파일러는 clone()을 제안하지만, 읽기만 하는 함수에서 문자열을 복사할 이유는 없습니다.
// 고치는 방법: as_deref()로 Option<&str>을 만들어 그것에 ?를 붙입니다.
//   let no = self.tracking_no.as_deref()?;
// (examples/04_borrowed_transforms.rs의 carrier 메서드)

struct Parcel {
    tracking_no: Option<String>,
}

impl Parcel {
    fn tracking_len(&self) -> Option<usize> {
        let no = self.tracking_no?;
        Some(no.chars().count())
    }
}

fn main() {
    let parcel = Parcel {
        tracking_no: Some(String::from("CJ2026111901")),
    };
    println!("{:?}", parcel.tracking_len());
}
