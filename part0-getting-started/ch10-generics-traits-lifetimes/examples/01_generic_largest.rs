// 이 예제가 보여 주는 것: 중복 코드를 함수로 추출하고, 타입만 다른 함수 두 개를 제네릭 함수 하나로 합치는 과정
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 01_generic_largest

// 1단계: 함수 추출. 목록마다 반복문을 복사해 붙이는 대신, "가장 큰 값 찾기"를 함수 하나로 뽑아냅니다.
// 매개변수를 슬라이스 &[i32]로 받으면 배열·벡터·그 일부를 모두 넘길 수 있습니다(ch4).
fn largest_i32(list: &[i32]) -> &i32 {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}

// 2단계: 그런데 char 목록에도 같은 일을 하려면 함수를 또 만들어야 합니다.
// 본문은 largest_i32와 한 글자도 다르지 않고, 시그니처의 타입만 다릅니다.
fn largest_char(list: &[char]) -> &char {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}

// 3단계: 제네릭 함수. 함수 이름 뒤 <T>는 "이 함수는 어떤 타입 T에 대해 동작한다"는 선언이고,
// 매개변수와 반환 타입의 i32·char 자리에 T를 씁니다.
//
// `T: PartialOrd`는 트레이트 바운드(trait bound)입니다. "T는 >, < 로 비교할 수 있는 타입이어야 한다"는 조건입니다.
// 이 조건이 없으면 직접 만든 구조체처럼 비교할 수 없는 타입도 T가 될 수 있으므로,
// 컴파일러는 본문의 `item > largest`를 거부합니다(compile_fail/e0369_largest_without_bound.rs).
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}

fn main() {
    println!("1. 타입마다 따로 만든 함수");
    let number_list = vec![34, 50, 25, 100, 65];
    let char_list = vec!['y', 'm', 'a', 'q'];
    println!("    largest_i32  → {}", largest_i32(&number_list));
    println!("    largest_char → {}", largest_char(&char_list));
    // 출력:
    //     largest_i32  → 100
    //     largest_char → y

    println!("2. 제네릭 함수 하나로 같은 일을 합니다");
    // 호출할 때 T를 따로 적지 않아도, 컴파일러가 인자의 타입을 보고 T를 정합니다.
    println!("    largest(&number_list) → {}", largest(&number_list)); // T = i32
    println!("    largest(&char_list)   → {}", largest(&char_list)); // T = char
    // 출력:
    //     largest(&number_list) → 100
    //     largest(&char_list)   → y

    println!("3. PartialOrd를 구현한 타입이라면 무엇이든 넘길 수 있습니다");
    let prices = [3.5, 12.25, 7.0]; // T = f64
    let words = ["사과", "바나나", "체리"]; // T = &str. 문자열은 사전순(유니코드 코드 포인트 순)으로 비교됩니다.
    println!("    가장 큰 실수: {}", largest(&prices));
    println!("    사전순으로 가장 뒤: {}", largest(&words));
    // 출력:
    //     가장 큰 실수: 12.25
    //     사전순으로 가장 뒤: 체리

    println!("4. 반환값은 목록 안 원소의 참조입니다");
    // 반환값이 복사본이 아니라 목록 안의 원소를 가리키므로, 원소의 위치를 비교해 확인할 수 있습니다.
    // std::ptr::eq는 두 참조가 같은 메모리를 가리키는지 비교합니다.
    let biggest = largest(&number_list);
    println!(
        "    number_list[3]을 가리키는가? {}",
        std::ptr::eq(biggest, &number_list[3])
    );
    // 출력: number_list[3]을 가리키는가? true
}
