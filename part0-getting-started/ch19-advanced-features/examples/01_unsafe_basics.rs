// unsafe의 기초: 원시 포인터를 만들고 역참조하기, unsafe 함수 정의와 호출,
// unsafe 코드를 안전한 함수로 감싸기(split_at_mut 직접 구현)를 보여 줍니다.
//
// 실행: cargo run -p p0-ch19-advanced-features --example 01_unsafe_basics
//
// 메모리 주소를 출력하는 줄은 실행할 때마다 값이 달라집니다.

use std::slice;

fn main() {
    // 1. 원시 포인터는 안전한 코드에서도 만들 수 있습니다.
    //    만들기만 해서는 아무 일도 일어나지 않기 때문입니다. 위험한 것은 따라가는(역참조하는) 순간입니다.
    println!("1. 원시 포인터 만들기");
    let mut num = 5;
    let r1 = &raw const num; // *const i32: 읽기 전용 원시 포인터
    let r2 = &raw mut num; // *mut i32: 쓰기도 할 수 있는 원시 포인터
    // 참조라면 불변 참조(r1)와 가변 참조(r2)를 동시에 만들 수 없습니다(ch4).
    // 원시 포인터는 대여 규칙의 검사를 받지 않으므로 컴파일됩니다.
    // 대신 둘을 섞어 써도 괜찮은지는 컴파일러가 아니라 프로그래머가 따져야 합니다.
    println!("    r1 = {r1:?}, r2 = {r2:?}"); // 출력: 실행마다 다른 주소 (두 값은 서로 같음)
    println!("    같은 곳을 가리키는가? {}", r1 == r2.cast_const()); // 출력:     같은 곳을 가리키는가? true

    // 2. 역참조는 unsafe 블록 안에서만 할 수 있습니다.
    //    *const 포인터로는 쓸 수 없습니다. `*r1 = 7;`은 unsafe 블록 안에서도 E0594입니다.
    println!("2. 원시 포인터 역참조");
    unsafe {
        // SAFETY: r2는 아직 살아 있는 지역 변수 num을 가리키고,
        // 이 블록에서는 r2 하나로만 num을 읽고 씁니다.
        *r2 += 1;
        println!("    *r2 = {}", *r2); // 출력:     *r2 = 6
    }
    println!("    num = {num}"); // 출력:     num = 6

    // 원시 포인터는 아무 데도 가리키지 않을(null) 수도 있습니다.
    // 역참조하기 전에 확인하는 것은 프로그래머의 몫입니다.
    let nothing: *const i32 = std::ptr::null();
    println!(
        "    null인가? r1: {}, nothing: {}",
        r1.is_null(),
        nothing.is_null()
    ); // 출력:     null인가? r1: false, nothing: true

    // 3. unsafe 함수: 부르는 쪽이 지켜야 할 조건이 있는 함수입니다.
    println!("3. unsafe 함수 호출");
    let numbers = [10, 20, 30];
    let first = numbers.as_ptr();
    // SAFETY: numbers의 길이가 3이므로 0, 1, 2번 위치는 모두 배열 안입니다.
    let second = unsafe { read_at(first, 1) };
    println!("    read_at(first, 1) = {second}"); // 출력:     read_at(first, 1) = 20

    // 표준 라이브러리에도 unsafe 함수가 있습니다. get_unchecked는 범위 검사를 건너뜁니다.
    // SAFETY: 2는 numbers의 길이(3)보다 작습니다.
    let third = unsafe { numbers.get_unchecked(2) };
    println!("    numbers.get_unchecked(2) = {third}"); // 출력:     numbers.get_unchecked(2) = 30

    // 4. 안전한 추상화: 안에서는 unsafe를 쓰지만 겉으로는 안전한 함수입니다.
    println!("4. 직접 만든 split_at_mut");
    let mut values = vec![1, 2, 3, 4, 5, 6];
    let (left, right) = split_at_mut(&mut values, 3);
    // 두 가변 슬라이스를 동시에 쓸 수 있습니다. 겹치지 않는다는 것을 함수가 보장하기 때문입니다.
    left[0] = 100;
    right[0] = 400;
    println!("    left = {left:?}, right = {right:?}"); // 출력:     left = [100, 2, 3], right = [400, 5, 6]
    println!("    values = {values:?}"); // 출력:     values = [100, 2, 3, 400, 5, 6]

    // 표준 라이브러리의 split_at_mut도 같은 방식으로 만들어져 있습니다.
    let (a, b) = values.split_at_mut(2);
    a[1] = b[0] * 10;
    println!("    표준 split_at_mut 사용 후 values = {values:?}"); // 출력:     표준 split_at_mut 사용 후 values = [100, 30, 3, 400, 5, 6]
}

/// `ptr`에서 `offset`칸 떨어진 `i32`를 읽습니다.
///
/// # Safety
///
/// `ptr`에서 `offset`칸 떨어진 위치가 살아 있는 `i32` 배열 안이어야 합니다.
unsafe fn read_at(ptr: *const i32, offset: usize) -> i32 {
    // Edition 2024에서는 unsafe 함수의 본문 안에서도 unsafe 연산을 unsafe 블록으로 감싸야 합니다.
    // 블록을 빼면 unsafe_op_in_unsafe_fn 경고가 납니다(README 2.3절).
    // SAFETY: 위치가 배열 안이라는 것은 부르는 쪽이 보장합니다(# Safety).
    unsafe { *ptr.add(offset) }
}

/// 슬라이스를 `mid` 위치에서 겹치지 않는 두 가변 슬라이스로 나눕니다.
/// unsafe 함수가 아니므로 부르는 쪽은 unsafe 블록이 필요 없습니다.
fn split_at_mut(values: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = values.len();
    let ptr = values.as_mut_ptr();

    // 이 검사가 아래 unsafe 블록을 안전하게 만드는 근거입니다.
    // mid가 len보다 크면 unsafe 코드에 들어가기 전에 패닉으로 멈춥니다.
    assert!(mid <= len);

    // SAFETY: mid <= len이므로 [0, mid)와 [mid, len)은 모두 values 안에 있고 서로 겹치지 않습니다.
    // 두 슬라이스의 라이프타임은 values의 대여에 묶이므로 values보다 오래 살 수 없습니다.
    unsafe {
        (
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}
