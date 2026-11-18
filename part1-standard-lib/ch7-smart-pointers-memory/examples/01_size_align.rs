// 이 예제가 보여 주는 것: 타입의 크기와 메모리 정렬, 구조체의 패딩과 필드 순서,
//                        크기가 0인 타입, size_of_val로 재는 "가리키는 값"의 크기
// 실행: cargo run -p p1-ch7-smart-pointers-memory --example 01_size_align
//
// 크기와 메모리 정렬 값은 64비트 환경(aarch64 macOS), rustc 1.100.0에서 확인한 것입니다.
// 원시 타입의 크기는 문서가 보장하지만, 기본 표현 구조체의 필드 배치는 보장되지 않으므로
// 컴파일러 버전이 바뀌면 3번 구역의 오프셋이 달라질 수 있습니다.

use std::mem::offset_of;

// 필드 크기의 합은 1 + 4 + 1 = 6바이트입니다.
struct Packet {
    tag: u8,
    id: u32,
    flag: u8,
}

// 필드 크기의 합은 1 + 8 + 2 = 11바이트입니다.
struct Record {
    kind: u8,
    timestamp: u64,
    code: u16,
}

// 필드가 없는 유닛 구조체. 담을 정보가 없으므로 크기도 0입니다.
struct Marker;

// 타입 T의 크기와 메모리 정렬을 한 줄로 출력합니다.
// size_of와 align_of는 프렐류드에 있어 use 없이 씁니다(Part 1 ch1).
fn layout<T>(name: &str) {
    println!(
        "{name:<12} 크기 {:>2}, 정렬 {:>2}",
        size_of::<T>(),
        align_of::<T>()
    );
}

fn main() {
    println!("== 1. 원시 타입의 크기와 메모리 정렬 ==");
    layout::<u8>("u8");
    layout::<u16>("u16");
    layout::<u32>("u32");
    layout::<u64>("u64");
    layout::<u128>("u128");
    layout::<char>("char");
    layout::<bool>("bool");
    layout::<usize>("usize");
    layout::<&u8>("&u8");
    // 출력:
    // u8           크기  1, 정렬  1
    // u16          크기  2, 정렬  2
    // u32          크기  4, 정렬  4
    // u64          크기  8, 정렬  8
    // u128         크기 16, 정렬 16
    // char         크기  4, 정렬  4
    // bool         크기  1, 정렬  1
    // usize        크기  8, 정렬  8
    // &u8          크기  8, 정렬  8

    // 메모리 정렬이 4라는 것은 "주소가 4의 배수인 곳에만 놓인다"는 뜻입니다.
    // 구조체는 필드 사이와 끝에 빈칸(패딩)을 넣어 모든 필드의 정렬을 맞춥니다.
    // 구조체의 정렬은 필드 중 가장 큰 정렬을 따르고, 크기는 그 정렬의 배수로 올림합니다.
    // 그래야 배열 [Packet; 3]에서 두 번째, 세 번째 원소도 4의 배수 주소에 놓입니다.
    println!("\n== 2. 구조체의 크기 = 필드 합 + 패딩 ==");
    layout::<Packet>("Packet");
    layout::<Record>("Record");
    layout::<[Packet; 3]>("[Packet; 3]");
    // 출력:
    // Packet       크기  8, 정렬  4
    // Record       크기 16, 정렬  8
    // [Packet; 3]  크기 24, 정렬  4

    // offset_of!(타입, 필드)는 구조체 시작에서 그 필드까지의 바이트 수입니다(1.77.0부터).
    // 선언 순서는 tag, id, flag지만, 컴파일러가 id를 맨 앞으로 옮겨 패딩을 줄였습니다.
    // 선언 순서대로 놓았다면 tag(1) + 빈칸(3) + id(4) + flag(1) + 빈칸(3) = 12바이트가 됩니다.
    println!("\n== 3. 기본 표현에서는 컴파일러가 필드 순서를 바꿉니다 ==");
    println!(
        "Packet: tag@{}, id@{}, flag@{}",
        offset_of!(Packet, tag),
        offset_of!(Packet, id),
        offset_of!(Packet, flag)
    ); // 출력: Packet: tag@4, id@0, flag@5
    println!(
        "Record: kind@{}, timestamp@{}, code@{}",
        offset_of!(Record, kind),
        offset_of!(Record, timestamp),
        offset_of!(Record, code)
    ); // 출력: Record: kind@10, timestamp@0, code@8
    // Record는 순서를 바꿔도 16바이트입니다. 8 + 2 + 1 = 11을 정렬 8의 배수로 올리면 16이기 때문입니다.
    // 튜플도 같은 규칙을 따릅니다.
    layout::<(u8, u32, u8)>("(u8,u32,u8)"); // 출력: (u8,u32,u8)  크기  8, 정렬  4

    // 담을 정보가 없는 타입은 메모리를 차지하지 않습니다. 몇 개를 모아도 0바이트입니다.
    println!("\n== 4. 크기가 0인 타입 ==");
    layout::<()>("()");
    layout::<Marker>("Marker");
    layout::<[Marker; 1000]>("[Marker; 1000]");
    layout::<[i32; 0]>("[i32; 0]");
    // 출력:
    // ()           크기  0, 정렬  1
    // Marker       크기  0, 정렬  1
    // [Marker; 1000] 크기  0, 정렬  1
    // [i32; 0]     크기  0, 정렬  4

    // size_of::<T>()는 타입만 보고 컴파일할 때 정해집니다.
    // size_of_val(&값)은 참조가 가리키는 값의 크기를 잽니다. 동적 크기 타입(str, [T])도 잴 수 있습니다.
    println!("\n== 5. size_of_val: 손잡이의 크기와 내용물의 크기 ==");
    let scores: Vec<i32> = vec![90, 85, 77];
    let text = String::from("안녕");
    println!(
        "Vec 손잡이 {}, 원소 슬라이스 {}",
        size_of_val(&scores),
        size_of_val(&scores[..])
    );
    // 출력: Vec 손잡이 24, 원소 슬라이스 12
    println!(
        "String 손잡이 {}, 글자 {}",
        size_of_val(&text),
        size_of_val(text.as_str())
    );
    // 출력: String 손잡이 24, 글자 6
    // Vec과 String 자체는 (포인터, 용량, 길이) 세 칸 24바이트짜리 손잡이이고,
    // 원소와 글자는 그 손잡이가 가리키는 힙에 있습니다. 힙 쪽 크기는 슬라이스로 재야 보입니다.

    // 필드를 실제로 읽어 dead_code 경고를 피합니다.
    let p = Packet {
        tag: 1,
        id: 7,
        flag: 0,
    };
    let r = Record {
        kind: 2,
        timestamp: 1_700_000_000,
        code: 404,
    };
    let markers = [Marker, Marker, Marker];
    println!(
        "\n필드 값: {} {} {} / {} {} {}, Marker {}개",
        p.tag,
        p.id,
        p.flag,
        r.kind,
        r.timestamp,
        r.code,
        markers.len()
    ); // 출력: 필드 값: 1 7 0 / 2 1700000000 404, Marker 3개
}
