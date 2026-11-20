// 지침 3: FFI에 쓰는 Rust 타입에는 #[repr(C)]를, 뉴타입에는 #[repr(transparent)]를 붙입니다.
//
// 실행: cargo run -p er-ch34-ffi-boundaries --example 02_repr_c_layout
//
// 크기와 오프셋은 64비트 플랫폼, 이 툴체인(rustc 1.100.0)에서 확인한 값입니다.
// 기본 표현(#[repr] 없음)의 배치는 언어가 약속하지 않으므로 컴파일러 버전에 따라 달라질 수 있습니다.

use std::mem::{align_of, offset_of, size_of};

// 센서 장치의 C 헤더가 이렇게 정의했다고 합시다.
//   struct reading { uint8_t flags; int64_t value; uint16_t channel; };

// 비권장: #[repr]이 없으면 Rust가 필드 순서를 마음대로 정합니다.
struct ReadingRust {
    flags: u8,
    value: i64,
    channel: u16,
}

// 권장: #[repr(C)]는 C 컴파일러와 같은 규칙(선언 순서, 각 필드의 메모리 정렬)으로 배치합니다.
#[repr(C)]
struct ReadingC {
    flags: u8,
    value: i64,
    channel: u16,
}

// 뉴타입 두 가지. 단위를 타입으로 구분하려고 f64를 감쌌습니다(ch6).
struct CelsiusPlain(f64);

#[repr(transparent)]
struct Celsius(f64);

// 비권장: 기본 표현의 뉴타입을 extern "C" 함수에 쓰면 improper_ctypes_definitions 경고가 납니다.
// (아래 #[expect]를 지우면 보입니다.) 이 환경에서 크기가 f64와 같아도 약속된 것은 아닙니다.
#[expect(
    improper_ctypes_definitions,
    reason = "repr이 없는 뉴타입을 extern \"C\" 함수에 쓰면 경고가 난다는 것을 보입니다"
)]
extern "C" fn warmer_plain(t: CelsiusPlain) -> CelsiusPlain {
    CelsiusPlain(t.0 + 1.5)
}

// 권장: #[repr(transparent)]는 "안쪽 f64와 같은 배치, 같은 호출 규약"을 약속합니다.
// C 헤더에는 그냥 double로 적으면 됩니다.
extern "C" fn warmer(t: Celsius) -> Celsius {
    Celsius(t.0 + 1.5)
}

// C의 enum은 정수로 받습니다. 헤더가 다음과 같다고 합시다.
//   enum sensor_state { SENSOR_IDLE = 0, SENSOR_RUNNING = 1, SENSOR_FAULT = 2 };
#[derive(Debug, PartialEq, Eq)]
enum SensorState {
    Idle,
    Running,
    Fault,
}

impl TryFrom<i32> for SensorState {
    type Error = i32;
    // 모르는 값은 Err로 돌려줍니다. C가 새 상태를 더해도 미정의 동작이 아니라 에러가 됩니다.
    fn try_from(raw: i32) -> Result<Self, i32> {
        match raw {
            0 => Ok(SensorState::Idle),
            1 => Ok(SensorState::Running),
            2 => Ok(SensorState::Fault),
            other => Err(other),
        }
    }
}

fn main() {
    println!("1. 같은 필드, 다른 표현");
    println!(
        "   기본 표현:  크기 {}, 정렬 {}, 오프셋 flags={} value={} channel={}",
        size_of::<ReadingRust>(),
        align_of::<ReadingRust>(),
        offset_of!(ReadingRust, flags),
        offset_of!(ReadingRust, value),
        offset_of!(ReadingRust, channel)
    );
    println!(
        "   #[repr(C)]: 크기 {}, 정렬 {}, 오프셋 flags={} value={} channel={}",
        size_of::<ReadingC>(),
        align_of::<ReadingC>(),
        offset_of!(ReadingC, flags),
        offset_of!(ReadingC, value),
        offset_of!(ReadingC, channel)
    );
    // 출력:    기본 표현:  크기 16, 정렬 8, 오프셋 flags=10 value=0 channel=8
    // 출력:    #[repr(C)]: 크기 24, 정렬 8, 오프셋 flags=0 value=8 channel=16
    // C 컴파일러는 #[repr(C)]와 같은 24바이트로 배치합니다. 기본 표현을 C에 넘기면
    // C는 0번 바이트를 flags로 읽지만, Rust는 거기에 value의 첫 바이트를 두었습니다.

    println!("2. 뉴타입: 크기는 같아도 약속이 다름");
    println!(
        "   CelsiusPlain {}바이트, Celsius {}바이트, f64 {}바이트",
        size_of::<CelsiusPlain>(),
        size_of::<Celsius>(),
        size_of::<f64>()
    );
    // 출력:    CelsiusPlain 8바이트, Celsius 8바이트, f64 8바이트
    println!(
        "   warmer_plain(20.0) = {}, warmer(20.0) = {}",
        warmer_plain(CelsiusPlain(20.0)).0,
        warmer(Celsius(20.0)).0
    );
    // 출력:    warmer_plain(20.0) = 21.5, warmer(20.0) = 21.5
    // 지금은 둘 다 동작합니다. 차이는 컴파일러가 무엇을 약속하느냐에 있고,
    // 약속이 없는 쪽에는 경고가 납니다.

    println!("3. C의 enum 값을 Rust 열거형으로 바꾸기");
    for raw in [1, 2, 7] {
        println!("   {raw} → {:?}", SensorState::try_from(raw));
    }
    // 출력:    1 → Ok(Running)
    // 출력:    2 → Ok(Fault)
    // 출력:    7 → Err(7)
}
