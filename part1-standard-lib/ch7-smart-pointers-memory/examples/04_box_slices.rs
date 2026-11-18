// 이 예제가 보여 주는 것: 얇은 포인터와 넓은 포인터의 크기, Box<[T]>와 Box<str>,
//                        큰 배리언트를 Box로 감싸 열거형을 작게 만들기, 큰 배열을 힙에 바로 만들기
// 실행: cargo run -p p1-ch7-smart-pointers-memory --example 04_box_slices
//
// 크기는 64비트 환경, rustc 1.100.0에서 확인한 값입니다.

use std::fmt::Display;

// Ping은 0바이트인데 Data가 1024바이트라, Ping 하나를 담아도 1KB 넘게 차지합니다.
// clippy는 이런 열거형에 large_enum_variant 경고를 냅니다. 비교하려고 일부러 만든 것이라 expect로 표시합니다.
#[expect(
    clippy::large_enum_variant,
    reason = "Box로 감싼 버전과 크기를 비교하려고 일부러 크게 만들었습니다"
)]
enum Message {
    Ping,
    Data([u8; 1024]),
}

// 큰 배리언트를 Box로 감싸면 열거형에는 포인터 한 칸만 남습니다.
// Box는 0이 될 수 없으므로 Ping은 "포인터 칸이 0"으로 나타내 태그 칸도 필요 없습니다(02 예제).
enum BoxedMessage {
    Ping,
    Data(Box<[u8; 1024]>),
}

fn payload_len(m: &Message) -> usize {
    match m {
        Message::Ping => 0,
        Message::Data(bytes) => bytes.len(),
    }
}

fn boxed_payload_len(m: &BoxedMessage) -> usize {
    match m {
        BoxedMessage::Ping => 0,
        BoxedMessage::Data(bytes) => bytes.len(),
    }
}

fn show<T>(name: &str) {
    println!("{name:<18} {:>4}바이트", size_of::<T>());
}

fn main() {
    // 크기를 컴파일할 때 아는 값을 가리키면 주소 한 칸(얇은 포인터)이고,
    // 동적 크기 타입을 가리키면 주소 + 추가 정보 두 칸(넓은 포인터)입니다.
    // 슬라이스와 str은 길이를, 트레이트 객체는 vtable 주소를 함께 듭니다(Part 0 ch17, ch19).
    println!("== 1. 얇은 포인터와 넓은 포인터 ==");
    show::<&i32>("&i32");
    show::<Box<i32>>("Box<i32>");
    show::<&[i32]>("&[i32]");
    show::<Box<[i32]>>("Box<[i32]>");
    show::<&str>("&str");
    show::<Box<str>>("Box<str>");
    show::<&dyn Display>("&dyn Display");
    show::<Box<dyn Display>>("Box<dyn Display>");
    show::<Vec<i32>>("Vec<i32>");
    show::<String>("String");
    // 출력:
    // &i32                  8바이트
    // Box<i32>              8바이트
    // &[i32]               16바이트
    // Box<[i32]>           16바이트
    // &str                 16바이트
    // Box<str>             16바이트
    // &dyn Display         16바이트
    // Box<dyn Display>     16바이트
    // Vec<i32>             24바이트
    // String               24바이트
    // Box<[T]>는 (주소, 길이), Vec<T>는 (주소, 용량, 길이)입니다. 용량 칸 하나만큼 차이가 납니다.

    println!("\n== 2. Box<[T]>: 더 늘지 않는 슬라이스를 소유하기 ==");
    let mut v: Vec<i32> = Vec::with_capacity(100);
    v.extend([10, 20, 30]);
    println!("Vec: 길이 {}, 용량 {}", v.len(), v.capacity()); // 출력: Vec: 길이 3, 용량 100
    // into_boxed_slice는 남는 용량을 버리고(필요하면 재할당해) 정확히 길이만큼의 힙 버퍼를 만듭니다.
    let mut fixed: Box<[i32]> = v.into_boxed_slice();
    fixed[0] = 11; // 소유한 슬라이스이므로 원소를 고칠 수는 있습니다. push처럼 길이를 바꾸는 메서드는 없습니다.
    println!("Box<[i32]>: {fixed:?}, 길이 {}", fixed.len()); // 출력: Box<[i32]>: [11, 20, 30], 길이 3
    let back = fixed.into_vec(); // 다시 Vec으로. 복사 없이 같은 버퍼를 넘겨받습니다.
    println!("다시 Vec: 길이 {}, 용량 {}", back.len(), back.capacity()); // 출력: 다시 Vec: 길이 3, 용량 3

    // 반복자에서 바로 모을 수도 있습니다.
    let squares: Box<[u32]> = (1..=5).map(|x| x * x).collect();
    println!("squares = {squares:?}"); // 출력: squares = [1, 4, 9, 16, 25]

    println!("\n== 3. Box<str>: 바뀌지 않는 문자열을 소유하기 ==");
    let city: Box<str> = String::from("부산").into_boxed_str();
    let country: Box<str> = Box::from("대한민국"); // &str에서 바로 만들 수도 있습니다.
    println!("{city}, {country}, 글자 바이트 {}", city.len()); // 출력: 부산, 대한민국, 글자 바이트 6
    // 이름 1000개를 담는다면 손잡이만 따져도 String은 24000바이트, Box<str>은 16000바이트입니다.
    println!(
        "손잡이 1000개: String {}바이트, Box<str> {}바이트",
        size_of::<String>() * 1000,
        size_of::<Box<str>>() * 1000
    ); // 출력: 손잡이 1000개: String 24000바이트, Box<str> 16000바이트
    let mut owned: String = city.into_string(); // 고쳐야 할 때는 다시 String으로 바꿉니다.
    owned.push_str("광역시");
    println!("{owned}"); // 출력: 부산광역시

    println!("\n== 4. 큰 배리언트는 Box로 감싸기 ==");
    show::<Message>("Message");
    show::<BoxedMessage>("BoxedMessage");
    // 출력:
    // Message            1025바이트
    // BoxedMessage          8바이트
    let inbox: Vec<Message> = (0..1000).map(|_| Message::Ping).collect();
    let boxed_inbox: Vec<BoxedMessage> = (0..1000).map(|_| BoxedMessage::Ping).collect();
    println!(
        "Ping 1000개: Message {}바이트, BoxedMessage {}바이트",
        size_of_val(&inbox[..]),
        size_of_val(&boxed_inbox[..])
    ); // 출력: Ping 1000개: Message 1025000바이트, BoxedMessage 8000바이트
    let data = BoxedMessage::Data(Box::new([7; 1024]));
    println!(
        "페이로드 길이: {}, {}, {}",
        payload_len(&inbox[0]),
        boxed_payload_len(&boxed_inbox[0]),
        boxed_payload_len(&data)
    ); // 출력: 페이로드 길이: 0, 0, 1024
    let raw = Message::Data([1; 1024]);
    println!("Message::Data 길이: {}", payload_len(&raw)); // 출력: Message::Data 길이: 1024

    println!("\n== 5. 아주 큰 배열은 힙에 바로 만들기 ==");
    // Box::new(값)은 값을 먼저 만든 뒤 힙으로 옮기므로, 디버그 빌드에서 Box::new([0u8; 16 MiB])는
    // 스택을 넘칠 수 있습니다. vec!은 처음부터 힙에 만들고, into_boxed_slice로 Box<[u8]>를 얻습니다.
    const LEN: usize = 16 * 1024 * 1024;
    let big: Box<[u8]> = vec![0u8; LEN].into_boxed_slice();
    // 길이가 정해진 배열 타입이 필요하면 try_into로 바꿉니다. 길이가 다르면 Err로 원래 Box를 돌려줍니다.
    let array: Box<[u8; LEN]> = big.try_into().expect("길이가 LEN이므로 실패하지 않습니다");
    println!("Box<[u8; LEN]>: 원소 {}개, 첫 값 {}", array.len(), array[0]);
    // 출력: Box<[u8; LEN]>: 원소 16777216개, 첫 값 0
}
