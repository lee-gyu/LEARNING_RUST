// 기대하는 에러: error[E0204]: the trait `Copy` cannot be implemented for this type
// 핵심 메시지: this field does not implement `Copy`
//
// Copy는 "비트를 그대로 복사해도 안전하다"는 표시입니다. String은 힙 버퍼를 소유하므로,
// 비트만 복사하면 두 값이 같은 버퍼를 가리키고 둘 다 해제하려 합니다(이중 해제).
// 그래서 필드 하나라도 Copy가 아니면 구조체도 Copy가 될 수 없습니다.
// 해결: derive에서 Copy를 빼고 필요한 곳에서 .clone()을 부릅니다. 또는 name을 Copy인 타입(예: 정수 ID)으로 바꿉니다.
//
// 확인: rustc --edition 2024 compile_fail/e0204_copy_with_string_field.rs --out-dir <임시 디렉터리>

#[derive(Debug, Clone, Copy)]
struct Member {
    id: u32,
    name: String,
}

fn main() {
    let a = Member {
        id: 1,
        name: String::from("민수"),
    };
    let b = a;
    println!("{} {:?}", a.id, b);
}
