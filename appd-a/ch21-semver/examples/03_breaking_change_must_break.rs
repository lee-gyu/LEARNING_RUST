// 동작이 호환되지 않게 바뀌면 타입도 바꿔서, 옛 사용법이 컴파일되지 않게 만듭니다.
//
// 실행: cargo run -p er-ch21-semver --example 03_breaking_change_must_break
//
// 가상의 택배 라이브러리 `parcel`의 무인 보관함 예약 함수 `reserve`를 세 판으로 흉내 냅니다.
//   v1      : 1.x. 보관 기간 `hold: u32`는 "일" 단위입니다.
//   v2_bad  : 2.0.0(비권장). 같은 `hold: u32`를 "시간" 단위로 읽게 바꿨습니다.
//             메이저 버전은 올렸지만 시그니처가 그대로라, 사용자의 옛 코드가 경고 없이 컴파일됩니다.
//   v2_good : 2.0.0(권장). 기간을 `std::time::Duration`으로 받습니다. 단위가 타입에 들어 있으므로
//             옛 코드 `reserve(7, 3)`은 E0308로 컴파일되지 않고, 사용자는 고칠 곳을 빠짐없이 알게 됩니다.

use std::time::Duration;

/// 예약 결과. 세 판이 같은 모양을 쓰도록 바깥에 두었습니다.
pub struct Reservation {
    locker: u32,
    hold_hours: u64,
}

impl Reservation {
    pub fn summary(&self) -> String {
        format!("{}번 보관함, {}시간 보관", self.locker, self.hold_hours)
    }
}

/// parcel 1.x
mod v1 {
    use super::Reservation;

    /// `hold`는 보관 일수입니다.
    pub fn reserve(locker: u32, hold: u32) -> Reservation {
        Reservation {
            locker,
            hold_hours: u64::from(hold) * 24,
        }
    }
}

/// parcel 2.0.0(비권장): 단위만 바꾸고 타입은 그대로 둡니다.
mod v2_bad {
    use super::Reservation;

    /// `hold`는 보관 시간입니다. (1.x에서는 일수였습니다)
    pub fn reserve(locker: u32, hold: u32) -> Reservation {
        Reservation {
            locker,
            hold_hours: u64::from(hold),
        }
    }
}

/// parcel 2.0.0(권장): 단위를 타입에 담습니다.
mod v2_good {
    use super::Reservation;
    use std::time::Duration;

    pub fn reserve(locker: u32, hold: Duration) -> Reservation {
        Reservation {
            locker,
            hold_hours: hold.as_secs() / 3600,
        }
    }
}

/// 1.x 시절에 쓴 사용자 코드: 7번 보관함을 3일 동안 예약합니다.
macro_rules! user_code_1x {
    ($parcel:ident) => {
        $parcel::reserve(7, 3).summary()
    };
}

fn main() {
    println!("[1.x] 옛 코드: {}", user_code_1x!(v1));
    println!(
        "[2.0.0 비권장] 옛 코드가 그대로 컴파일됨: {}",
        user_code_1x!(v2_bad)
    );

    // user_code_1x!(v2_good)은 컴파일되지 않습니다(README 2.6절의 E0308).
    // 컴파일 에러가 가리킨 곳을 고친 코드입니다. 단위가 호출하는 곳에 드러납니다.
    let fixed = v2_good::reserve(7, Duration::from_hours(3 * 24)).summary();
    println!("[2.0.0 권장] 옛 코드는 E0308, 고친 코드: {fixed}");
    // 출력:
    // [1.x] 옛 코드: 7번 보관함, 72시간 보관
    // [2.0.0 비권장] 옛 코드가 그대로 컴파일됨: 7번 보관함, 3시간 보관
    // [2.0.0 권장] 옛 코드는 E0308, 고친 코드: 7번 보관함, 72시간 보관
}
