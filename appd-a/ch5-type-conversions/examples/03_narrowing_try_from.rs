// 지침 3: 좁히는 변환은 `as` 대신 `try_from`으로 합니다.
//
// 묶음 파일(archive)의 목록 헤더를 씁니다. 헤더 형식은 파일 크기와 수정 시각을 `u32`로 저장하지만,
// 프로그램 안에서는 크기를 `u64`, 수정 시각을 `i64`(유닉스 시간, 초)로 다룹니다.
// 두 변환 모두 값이 `u32`에 들어가지 않을 수 있는 좁히는 변환입니다.
// 비권장: `as u32`로 맞춥니다. 들어가지 않는 값은 아무 알림 없이 다른 값이 됩니다.
// 권장:   `u32::try_from`으로 바꾸고, 들어가지 않는 항목은 에러로 알립니다.
// 4번 구역은 `TryFrom`을 쓸 수 없어 `as`가 맞는 자리(실수 → 정수, const)를 보여 줍니다.
//
// 실행: cargo run -p er-ch5-type-conversions --example 03_narrowing_try_from

use std::num::TryFromIntError;

struct Entry {
    name: &'static str,
    size: u64,     // 바이트
    modified: i64, // 1970-01-01 00:00 UTC부터 지난 초. 그 이전이면 음수입니다.
}

const fn entry(name: &'static str, size: u64, modified: i64) -> Entry {
    Entry {
        name,
        size,
        modified,
    }
}

const ENTRIES: [Entry; 3] = [
    entry("notes.txt", 12_800, 1_700_000_000),
    entry("movie.mkv", 5_000_000_000, 1_700_000_000), // 약 4.7 GiB
    entry("scan.tif", 1_200, -86_400),                // 1969-12-31
];

/// 헤더에 기록되는 모양입니다.
struct Header {
    size: u32,
    modified: u32,
}

fn show(name: &str, h: &Header) {
    println!(
        "    {name:<10} size={:>10} modified={:>10}",
        h.size, h.modified
    );
}

/// 비권장: `as`는 실패하지 않습니다. 대신 들어가지 않는 값을 다른 값으로 바꿉니다.
fn header_with_as(e: &Entry) -> Header {
    Header {
        size: e.size as u32,         // 위쪽 32비트를 버립니다.
        modified: e.modified as u32, // 음수는 비트 그대로 읽혀 아주 큰 양수가 됩니다.
    }
}

/// 어느 필드에서 실패했는지 담는 에러입니다. 에러 타입의 설계는 ch4에서 다룹니다.
enum HeaderError {
    Size(TryFromIntError),
    Modified(TryFromIntError),
}

/// 권장: 들어가지 않는 값이면 `Err`를 돌려줍니다. 실패할 수 있다는 사실이 시그니처에 드러납니다.
fn header(e: &Entry) -> Result<Header, HeaderError> {
    let size = u32::try_from(e.size).map_err(HeaderError::Size)?;
    let modified = u32::try_from(e.modified).map_err(HeaderError::Modified)?;
    Ok(Header { size, modified })
}

/// 진행률(%)을 정수로 돌려줍니다. 표준 라이브러리에는 실수 → 정수의 `TryFrom`이 없으므로 `as`를 씁니다.
/// `as`는 소수점 아래를 버리고, 범위 밖이면 끝값에 붙이고, NaN은 0으로 만듭니다.
/// 그래서 반올림 방식과 "계산할 수 없는 경우"는 `as` 앞에서 직접 정합니다.
fn percent(done: u32, total: u32) -> Option<u8> {
    if total == 0 {
        return None; // 0.0 / 0.0은 NaN이고, NaN as u8은 조용히 0이 됩니다.
    }
    let ratio = f64::from(done) / f64::from(total);
    Some((ratio * 100.0).round().clamp(0.0, 100.0) as u8)
}

// const 문맥에서는 `From::from`과 `try_from`을 부를 수 없습니다(1.100.0에서 `From`은 const 트레이트가 아님).
// 그래서 `as`를 쓰되, 값이 들어가는지는 컴파일할 때 단언으로 확인합니다.
const BLOCK_SIZE: u64 = 4096;
const _: () = assert!(BLOCK_SIZE <= u32::MAX as u64);
const BLOCK_SIZE_U32: u32 = BLOCK_SIZE as u32;

fn main() {
    println!("1. 비권장: as u32로 헤더를 씁니다");
    for e in &ENTRIES {
        show(e.name, &header_with_as(e));
    }
    // 출력:     notes.txt  size=     12800 modified=1700000000
    // 출력:     movie.mkv  size= 705032704 modified=1700000000
    // 출력:     scan.tif   size=      1200 modified=4294880896

    println!("2. 권장: u32::try_from으로 헤더를 씁니다");
    for e in &ENTRIES {
        match header(e) {
            Ok(h) => show(e.name, &h),
            Err(HeaderError::Size(err)) => println!("    {:<10} 거부(size): {err}", e.name),
            Err(HeaderError::Modified(err)) => println!("    {:<10} 거부(modified): {err}", e.name),
        }
    }
    // 출력:     notes.txt  size=     12800 modified=1700000000
    // 출력:     movie.mkv  거부(size): number too large to fit in target type
    // 출력:     scan.tif   거부(modified): number too small to fit in target type

    println!("3. 비권장 헤더를 다시 읽으면");
    // 잘린 값은 헤더만 봐서는 정상 값과 구별할 수 없습니다.
    let movie = header_with_as(&ENTRIES[1]);
    let lost = ENTRIES[1].size - u64::from(movie.size);
    println!("    movie.mkv: {} 바이트를 잃었습니다", lost); // 출력:     movie.mkv: 4294967296 바이트를 잃었습니다
    let scan = header_with_as(&ENTRIES[2]);
    let years = i64::from(scan.modified) / (365 * 24 * 60 * 60);
    println!("    scan.tif: 1970년부터 약 {years}년 뒤의 파일로 기록됐습니다"); // 출력:     scan.tif: 1970년부터 약 136년 뒤의 파일로 기록됐습니다

    println!("4. as가 맞는 자리: TryFrom이 없는 변환");
    println!("    진행률 3/8 = {:?}", percent(3, 8)); // 출력:     진행률 3/8 = Some(38)
    println!("    진행률 0/0 = {:?}", percent(0, 0)); // 출력:     진행률 0/0 = None
    println!("    블록 크기 = {BLOCK_SIZE_U32}"); // 출력:     블록 크기 = 4096
}
