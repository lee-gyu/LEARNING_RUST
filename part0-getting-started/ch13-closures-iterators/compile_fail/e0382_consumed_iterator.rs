// 기대하는 에러: error[E0382]: use of moved value: `v1_iter`
// 핵심 메시지: `v1_iter` moved due to this method call / value used here after move
//             note: `std::iter::Iterator::sum` takes ownership of the receiver `self`, which moves `v1_iter`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0382_consumed_iterator.rs --out-dir /tmp/rust-ch13
//
// sum 같은 소비 메서드는 self를 받아 반복자의 소유권을 가져가고, next를 끝까지 불러 다 써 버립니다.
// 그래서 sum을 부른 뒤에는 v1_iter를 다시 쓸 수 없습니다. 설령 쓸 수 있다 해도 이미 끝까지 간
// 반복자라서 더 내놓을 값이 없습니다.
// 고치는 방법: 결과가 여러 개 필요하면 원본(v1)에서 반복자를 새로 만듭니다: v1.iter().count()

fn main() {
    let v1 = vec![1, 2, 3];

    let v1_iter = v1.iter();
    let total: i32 = v1_iter.sum();
    let count = v1_iter.count();

    println!("합: {total}, 개수: {count}");
}
