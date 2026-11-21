// Q4.10 · Q4.9 — 값은 기본으로 그 자리에 놓이고, 힙은 고를 때만 쓴다
//
// Rust의 구조체와 배열은 필드와 원소를 그 자리에 그대로 담습니다. 구조체 안의 구조체도, `Vec` 안의
// 구조체도 따로 할당하지 않고 나란히 붙여 둡니다. 힙에 따로 두려면 `Box`처럼 그렇다고 적어야 합니다.
//
// 크기와 간격은 64비트 플랫폼 기준입니다. 주소 값 자체는 실행마다 다르므로 출력하지 않고,
// 두 원소 주소의 차이만 출력합니다.
//
// 실행: cargo run -p faq-ch4-memory-without-gc --example 03_values_inline

use std::mem::size_of;

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

struct Segment {
    start: Point,
    end: Point,
}

impl Segment {
    fn length(&self) -> f64 {
        (self.end.x - self.start.x).hypot(self.end.y - self.start.y)
    }
}

/// i번째 점: (0, 0), (3, 4), (6, 8)
fn point(i: u32) -> Point {
    let i = f64::from(i);
    Point {
        x: 3.0 * i,
        y: 4.0 * i,
    }
}

/// 같은 슬라이스 안에서 이웃한 두 원소의 주소 차이(바이트)
fn stride<T>(items: &[T]) -> usize {
    (&items[1] as *const T).addr() - (&items[0] as *const T).addr()
}

fn main() {
    println!("1. 구조체 안의 구조체는 그 자리에 담긴다");
    println!(
        "  Point {}바이트, Segment {}바이트",
        size_of::<Point>(),
        size_of::<Segment>()
    );
    // 출력:   Point 16바이트, Segment 32바이트
    // Segment는 Point 둘을 가리키는 주소 둘이 아니라, Point 둘 자체(f64 넷)입니다.

    println!("2. Vec<Point>: 원소가 힙에 나란히 붙어 있다");
    let points: Vec<Point> = (0..3).map(point).collect();
    println!("  이웃한 원소의 간격 {}바이트", stride(&points));
    // 출력:   이웃한 원소의 간격 16바이트
    // 버퍼 하나에 Point가 16바이트씩 이어져 있어, 훑을 때 메모리를 차례로 읽습니다.

    println!("3. Vec<Box<Point>>: 원소마다 따로 할당한 값을 가리킨다");
    let boxed: Vec<Box<Point>> = (0..3).map(|i| Box::new(point(i))).collect();
    println!(
        "  원소 하나의 크기 {}바이트, 이웃한 원소의 간격 {}바이트",
        size_of::<Box<Point>>(),
        stride(&boxed)
    );
    // 출력:   원소 하나의 크기 8바이트, 이웃한 원소의 간격 8바이트
    // 버퍼에는 주소만 이어져 있고, Point는 각자 다른 곳에 따로 할당되어 있습니다.

    let segment = Segment {
        start: points[0], // Copy 타입이라 Vec 안의 값을 복사해 옵니다.
        end: *boxed[1],   // Box가 가리키는 값을 복사해 옵니다.
    };
    println!("  (0, 0)에서 (3, 4)까지의 길이 {}", segment.length());
    // 출력:   (0, 0)에서 (3, 4)까지의 길이 5
}
