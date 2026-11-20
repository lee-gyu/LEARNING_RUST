// 지침 5: C 자원은 Drop을 구현한 래퍼에 담고, NULL과 상태 코드는 Result로 바꿉니다.
//
// 실행: cargo run -p er-ch34-ffi-boundaries --example 04_drop_wrapper
//
// fake_c 모듈이 C 라이브러리를 대신합니다(열린 핸들 수를 세어 누수를 볼 수 있게).
// 실제 C 라이브러리라면 unsafe extern 블록의 선언을 부르겠지만, 부르는 쪽 코드의 모양
// (unsafe 블록, NULL 검사, 해제 함수 호출)은 같습니다. 진짜 C 라이브러리를 감싼 판은
// projects/tally_ffi에 있습니다.

use std::fmt;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

/// C 라이브러리 흉내: "용량이 정해진 계량기"를 열고, 더하고, 닫습니다.
mod fake_c {
    use super::*;

    pub static OPEN: AtomicUsize = AtomicUsize::new(0);

    pub const METER_OK: i32 = 0;
    pub const METER_ERR_NULL: i32 = -1;
    pub const METER_ERR_FULL: i32 = -2;

    pub struct Meter {
        total: u32,
        capacity: u32,
    }

    /// capacity가 0이면 NULL을 돌려줍니다.
    pub extern "C" fn meter_open(capacity: u32) -> *mut Meter {
        if capacity == 0 {
            return std::ptr::null_mut();
        }
        OPEN.fetch_add(1, Ordering::SeqCst);
        Box::into_raw(Box::new(Meter { total: 0, capacity }))
    }

    /// # Safety
    /// `m`은 NULL이거나 meter_open이 돌려주고 닫지 않은 포인터여야 합니다.
    pub unsafe extern "C" fn meter_add(m: *mut Meter, amount: u32) -> i32 {
        // SAFETY: 이 함수의 전제 조건. NULL은 as_mut()이 None으로 바꿉니다.
        let Some(m) = (unsafe { m.as_mut() }) else {
            return METER_ERR_NULL;
        };
        match m.total.checked_add(amount) {
            Some(t) if t <= m.capacity => {
                m.total = t;
                METER_OK
            }
            _ => METER_ERR_FULL,
        }
    }

    /// # Safety
    /// `m`은 NULL이거나 meter_open이 돌려주고 닫지 않은 포인터여야 합니다.
    pub unsafe extern "C" fn meter_total(m: *const Meter) -> u32 {
        // SAFETY: 이 함수의 전제 조건
        unsafe { m.as_ref() }.map_or(0, |m| m.total)
    }

    /// # Safety
    /// `m`은 NULL이거나 meter_open이 돌려주고 닫지 않은 포인터여야 합니다.
    pub unsafe extern "C" fn meter_close(m: *mut Meter) {
        if m.is_null() {
            return;
        }
        OPEN.fetch_sub(1, Ordering::SeqCst);
        // SAFETY: m은 meter_open이 Box::into_raw로 만들었고 아직 닫지 않았다는 것이 전제 조건입니다.
        drop(unsafe { Box::from_raw(m) });
    }
}

fn open_handles() -> usize {
    fake_c::OPEN.load(Ordering::SeqCst)
}

#[derive(Debug)]
enum MeterError {
    OpenFailed,
    Full { amount: u32 },
    Status(i32),
}

impl fmt::Display for MeterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeterError::OpenFailed => write!(f, "계량기를 열 수 없습니다"),
            MeterError::Full { amount } => write!(f, "{amount}을(를) 더하면 용량을 넘습니다"),
            MeterError::Status(code) => write!(f, "알 수 없는 상태 코드 {code}"),
        }
    }
}

/// 비권장: 원시 포인터를 직접 들고 열고 닫습니다.
fn fill_manual(capacity: u32, amounts: &[u32]) -> Result<u32, MeterError> {
    let m = fake_c::meter_open(capacity);
    if m.is_null() {
        return Err(MeterError::OpenFailed);
    }
    for &amount in amounts {
        // SAFETY: m은 meter_open이 돌려준 NULL이 아닌 포인터이고 아직 닫지 않았습니다.
        let status = unsafe { fake_c::meter_add(m, amount) };
        if status != fake_c::METER_OK {
            return Err(MeterError::Full { amount }); // ← meter_close를 건너뜁니다
        }
    }
    // SAFETY: 위와 같습니다.
    let total = unsafe { fake_c::meter_total(m) };
    // SAFETY: 위와 같고, 여기서 한 번만 닫습니다.
    unsafe { fake_c::meter_close(m) };
    Ok(total)
}

/// 권장: 핸들을 소유하는 래퍼. 만들 때 NULL을 거르고, 버려질 때 닫습니다.
/// unsafe 블록은 모두 이 타입의 메서드 안에 있습니다.
struct Meter {
    raw: NonNull<fake_c::Meter>,
}

impl Meter {
    fn open(capacity: u32) -> Result<Meter, MeterError> {
        NonNull::new(fake_c::meter_open(capacity))
            .map(|raw| Meter { raw })
            .ok_or(MeterError::OpenFailed)
    }

    fn add(&mut self, amount: u32) -> Result<(), MeterError> {
        // SAFETY: self.raw는 meter_open이 돌려준 포인터이고 drop 전에는 닫히지 않습니다.
        match unsafe { fake_c::meter_add(self.raw.as_ptr(), amount) } {
            fake_c::METER_OK => Ok(()),
            fake_c::METER_ERR_FULL => Err(MeterError::Full { amount }),
            other => Err(MeterError::Status(other)), // METER_ERR_NULL은 여기로 오지 않습니다
        }
    }

    fn total(&self) -> u32 {
        // SAFETY: 위와 같습니다.
        unsafe { fake_c::meter_total(self.raw.as_ptr()) }
    }
}

impl Drop for Meter {
    fn drop(&mut self) {
        // SAFETY: self.raw는 meter_open이 돌려준 포인터이고, 닫는 곳은 여기 한 곳뿐입니다.
        unsafe { fake_c::meter_close(self.raw.as_ptr()) }
    }
}

fn fill_wrapped(capacity: u32, amounts: &[u32]) -> Result<u32, MeterError> {
    let mut m = Meter::open(capacity)?;
    for &amount in amounts {
        m.add(amount)?; // 여기서 빠져나가도 m이 버려지며 닫힙니다
    }
    Ok(m.total())
}

fn main() {
    let amounts = [30, 50, 40];

    println!("1. 비권장: 직접 열고 닫기");
    println!("   용량 200: {:?}", fill_manual(200, &amounts));
    println!("   열린 핸들: {}", open_handles());
    println!("   용량 100: {:?}", fill_manual(100, &amounts));
    println!("   열린 핸들: {}", open_handles());
    // 출력:    용량 200: Ok(120)
    // 출력:    열린 핸들: 0
    // 출력:    용량 100: Err(Full { amount: 40 })
    // 출력:    열린 핸들: 1
    // 실패한 경로에서 meter_close를 건너뛰어 핸들 하나가 새었습니다.

    println!("2. 권장: Drop 래퍼");
    let before = open_handles();
    println!("   용량 200: {:?}", fill_wrapped(200, &amounts));
    let result = fill_wrapped(100, &amounts);
    println!("   용량 100: {result:?}");
    if let Err(e) = &result {
        println!("   Display: {e}");
    }
    println!("   새로 남은 핸들: {}", open_handles() - before);
    // 출력:    용량 200: Ok(120)
    // 출력:    용량 100: Err(Full { amount: 40 })
    // 출력:    Display: 40을(를) 더하면 용량을 넘습니다
    // 출력:    새로 남은 핸들: 0

    println!("3. NULL을 Result로");
    println!("   Meter::open(0): {:?}", Meter::open(0).err());
    // 출력:    Meter::open(0): Some(OpenFailed)
}
