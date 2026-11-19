//! 공연 좌석을 예약하는 작은 프로그램입니다. 볼 것은 실행 결과보다 이 프로그램의 의존성 그래프입니다.

use derive_builder::Builder;

/// 공연 정보입니다. `ConcertBuilder`는 derive_builder가 만들어 줍니다(ch7).
#[derive(Debug, Builder)]
struct Concert {
    title: String,
    #[builder(default = "100")]
    seats: u32,
}

/// 좌석이 모자랄 때의 에러입니다. `Display`와 `Error` 구현은 thiserror가 만들어 줍니다(ch4).
#[derive(Debug, thiserror::Error)]
#[error("좌석이 모자랍니다: 남은 {left}석, 요청 {asked}석")]
struct SoldOut {
    left: u32,
    asked: u32,
}

/// 예약하고 남는 좌석 수를 돌려줍니다.
fn reserve(concert: &Concert, asked: u32) -> Result<u32, SoldOut> {
    concert.seats.checked_sub(asked).ok_or(SoldOut {
        left: concert.seats,
        asked,
    })
}

fn main() -> anyhow::Result<()> {
    let concert = ConcertBuilder::default()
        .title("겨울 음악회".to_string())
        .build()?;
    println!("{}: {}석", concert.title, concert.seats);
    println!("30석 예약 → 남은 좌석 {}석", reserve(&concert, 30)?);
    if let Err(e) = reserve(&concert, 500) {
        println!("500석 예약 → {e}");
    }
    Ok(())
}
