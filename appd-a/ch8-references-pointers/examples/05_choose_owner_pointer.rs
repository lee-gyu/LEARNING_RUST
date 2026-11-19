// 지침 5: 스마트 포인터는 질문 순서대로 고릅니다. 먼저 "누가 소유하는가"부터 답합니다.
//
//   ① 소유자가 하나인가?  → 값을 그대로 둡니다. 간접 참조가 꼭 필요할 때만 Box.
//   ② 소유자가 여럿인데 스레드를 넘는가? → 같은 스레드면 Rc, 넘으면 Arc.
//      빌려 쓰는 것으로 충분하면(스코프 스레드) 스마트 포인터 없이 &T.
//
// 나머지 두 질문(내부 가변성이 필요한가, 순환이 생기는가)은 예제 06에서 이어집니다.
//
// 실행: cargo run -p er-ch8-references-pointers --example 05_choose_owner_pointer

use std::ops::Range;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

#[derive(Clone)]
struct Photo {
    name: String,
    pixels: Vec<u8>,
}

impl Photo {
    fn new(name: &str, size: usize) -> Self {
        Photo {
            name: name.to_string(),
            pixels: (0..size).map(|i| (i % 251) as u8).collect(),
        }
    }

    /// 주어진 범위 픽셀의 평균 밝기
    fn brightness(&self, range: Range<usize>) -> u64 {
        let part = &self.pixels[range];
        part.iter().map(|&p| u64::from(p)).sum::<u64>() / part.len() as u64
    }
}

/// 비권장: 소유자가 하나뿐인 사진들을 하나씩 상자에 담습니다.
struct BoxedAlbum {
    #[expect(clippy::vec_box, reason = "지침 5의 비권장 형태를 보여 주는 코드")]
    photos: Vec<Box<Photo>>,
}

/// 권장: Vec이 이미 원소를 힙에 두므로 값을 그대로 담습니다.
struct Album {
    photos: Vec<Photo>,
}

/// `item`이 `buffer`의 메모리 안에 놓여 있는지 주소로만 확인합니다(역참조하지 않음).
fn lies_in<T, U>(buffer: &[T], item: &U) -> bool {
    let range = buffer.as_ptr_range();
    (range.start.addr()..range.end.addr()).contains(&ptr::from_ref(item).addr())
}

/// 여러 앨범에 든 사진들의 픽셀 버퍼가 실제로 몇 벌인지 셉니다(같은 주소면 한 벌).
fn distinct_buffers(photos: &[&Photo]) -> (usize, usize) {
    let mut seen: Vec<&[u8]> = Vec::new();
    for photo in photos {
        if !seen.iter().any(|s| ptr::eq(*s, photo.pixels.as_slice())) {
            seen.push(&photo.pixels);
        }
    }
    (seen.len(), seen.iter().map(|s| s.len()).sum())
}

fn main() {
    println!("1. 소유자가 하나: 값을 그대로 둡니다 (64비트 기준)");
    let boxed = BoxedAlbum {
        photos: vec![
            Box::new(Photo::new("바다", 16)),
            Box::new(Photo::new("숲", 16)),
        ],
    };
    let plain = Album {
        photos: vec![Photo::new("바다", 16), Photo::new("숲", 16)],
    };
    // Vec<Box<Photo>>의 버퍼에는 포인터만 있고, 사진은 각자 따로 할당된 곳에 있습니다.
    println!(
        "    비권장 Vec<Box<Photo>>: 버퍼 {}바이트, 사진이 버퍼 안에 있는가: {}",
        size_of_val(&boxed.photos[..]),
        lies_in(&boxed.photos, &*boxed.photos[1])
    );
    // 출력:     비권장 Vec<Box<Photo>>: 버퍼 16바이트, 사진이 버퍼 안에 있는가: false
    println!(
        "    권장 Vec<Photo>: 버퍼 {}바이트, 사진이 버퍼 안에 있는가: {}",
        size_of_val(&plain.photos[..]),
        lies_in(&plain.photos, &plain.photos[1])
    );
    // 출력:     권장 Vec<Photo>: 버퍼 96바이트, 사진이 버퍼 안에 있는가: true
    println!(
        "    두 앨범의 두 번째 사진: {} / {}",
        boxed.photos[1].name, plain.photos[1].name
    );
    // 출력:     두 앨범의 두 번째 사진: 숲 / 숲

    println!("2. 소유자가 여럿(같은 스레드): 복제 대신 Rc로 공유합니다");
    const SIZE: usize = 1_000_000; // 1MB짜리 사진
    // 비권장: 두 앨범이 각자 소유하도록 복제합니다. 픽셀 버퍼도 통째로 복제됩니다.
    let original = Photo::new("졸업식", SIZE);
    let family_copy = original.clone();
    let trip_copy = original; // 원본은 여행 앨범으로 옮깁니다.
    let (count, bytes) = distinct_buffers(&[&family_copy, &trip_copy]);
    println!("    복제: 픽셀 버퍼 {count}벌, {bytes}바이트");
    // 출력:     복제: 픽셀 버퍼 2벌, 2000000바이트
    // 권장: Rc로 감싸고 Rc::clone으로 소유자를 늘립니다. 복제되는 것은 포인터와 카운트뿐입니다.
    let shared = Rc::new(Photo::new("졸업식", SIZE));
    let family: Vec<Rc<Photo>> = vec![Rc::clone(&shared)];
    let trip: Vec<Rc<Photo>> = vec![Rc::clone(&shared)];
    let (count, bytes) = distinct_buffers(&[&family[0], &trip[0]]);
    println!(
        "    Rc: 픽셀 버퍼 {count}벌, {bytes}바이트, 소유자 {}명, 같은 사진인가: {}",
        Rc::strong_count(&shared),
        Rc::ptr_eq(&family[0], &trip[0])
    );
    // 출력:     Rc: 픽셀 버퍼 1벌, 1000000바이트, 소유자 3명, 같은 사진인가: true

    println!("3. 스레드를 넘는가: spawn이면 Arc, 스코프 스레드면 &T로 충분합니다");
    // Rc는 카운트를 원자적으로 바꾸지 않으므로 Send가 아니고, spawn에 넘기면 E0277입니다
    // (Part 0 ch15 4.4절). 스레드를 넘는 공유 소유에는 Arc를 씁니다.
    let photo = Arc::new(Photo::new("야경", SIZE));
    let half = SIZE / 2;
    let handles: Vec<_> = [0..half, half..SIZE]
        .into_iter()
        .map(|range| {
            let mine = Arc::clone(&photo); // spawn한 스레드는 'static이어야 하므로 소유권을 나눠 줍니다.
            thread::spawn(move || mine.brightness(range))
        })
        .collect();
    let by_arc: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!(
        "    spawn + Arc: 밝기 {by_arc:?}, 스레드가 끝난 뒤 소유자 {}명",
        Arc::strong_count(&photo)
    );
    // 출력:     spawn + Arc: 밝기 [124, 124], 스레드가 끝난 뒤 소유자 1명
    // 스코프 스레드는 스코프가 끝나기 전에 모두 합류하므로 지역 값을 빌려 쓸 수 있습니다.
    let local = Photo::new("야경", SIZE);
    let by_ref: Vec<u64> = thread::scope(|s| {
        let first = s.spawn(|| local.brightness(0..half));
        let second = s.spawn(|| local.brightness(half..SIZE));
        vec![first.join().unwrap(), second.join().unwrap()]
    });
    println!("    scope + &Photo: 밝기 {by_ref:?}, 카운트 없음");
    // 출력:     scope + &Photo: 밝기 [124, 124], 카운트 없음
}
