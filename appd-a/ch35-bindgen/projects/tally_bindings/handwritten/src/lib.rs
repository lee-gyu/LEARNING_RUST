//! C 라이브러리 tally의 안전한 래퍼입니다(손으로 쓴 선언 `ffi.rs` 위에 얹음).
//!
//! ch34의 `src/lib.rs`를 옮긴 것이고, 바뀐 곳은 셋입니다. 데모 출력용 `C_VERSION`과
//! `DECLARATIONS`를 더했고, `TallyError::Status`의 문구를 "래퍼가 모르는 상태 코드"로
//! 바꿨습니다. 생성한 선언 위의 래퍼(`generated/tally`)와 견줘 보면 선언의 출처가 바뀔 때
//! 래퍼의 어느 줄이 바뀌는지 알 수 있습니다.
//!
//! - unsafe 코드는 이 파일과 선언만 모은 `ffi.rs`에만 있습니다. 이 크레이트를 쓰는
//!   코드(데모, 테스트)는 unsafe 없이 씁니다.
//! - C가 할당한 것은 C가 짝지어 준 해제 함수로만, `Drop`에서 해제합니다.
//! - 이 파일의 unsafe 코드는 Miri로 확인하지 못했습니다(Miri는 C 코드를 실행하지 못합니다).
//!   블록마다 `// SAFETY:`로 근거를 적었습니다.

mod ffi;

use std::error::Error;
use std::ffi::{CStr, CString, c_char};
use std::fmt;
use std::ptr::NonNull;

/// 구간 수의 상한(C의 `TALLY_MAX_BUCKETS`)
pub const MAX_BUCKETS: u32 = ffi::TALLY_MAX_BUCKETS;
/// 이름의 최대 바이트 수(C의 `TALLY_NAME_MAX`)
pub const NAME_MAX: usize = ffi::TALLY_NAME_MAX;

/// 이 빌드가 링크한 C 소스의 판(`"1"`, `"2"`, `"3"`). 빌드 스크립트가 `TALLY_C_VERSION`에서 정합니다.
pub const C_VERSION: &str = env!("TALLY_C_VERSION");
/// 이 래퍼가 쓰는 선언의 출처
pub const DECLARATIONS: Declarations = Declarations::Handwritten;

/// 선언을 어디서 얻었는가(데모 출력용). `generated/tally`에도 같은 타입이 있습니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declarations {
    /// 사람이 헤더를 보고 손으로 옮긴 선언
    Handwritten,
    /// bindgen이 만든 결과를 저장소에 커밋해 둔 것
    Committed,
    /// 빌드할 때 이 C 소스의 헤더에서 bindgen이 새로 생성한 것
    Generated,
}

impl Declarations {
    /// 출력에 쓸 이름
    pub fn label(self) -> &'static str {
        match self {
            Declarations::Handwritten => "손으로 쓴 선언",
            Declarations::Committed => "커밋해 둔 생성 결과",
            Declarations::Generated => "빌드 때 생성한 선언",
        }
    }
}

/// 구간 설정. 구간 i는 `[low + i * bucket_width, low + (i + 1) * bucket_width)`입니다.
///
/// C의 `tally_config`와 내용은 같지만 Rust 쪽 타입입니다. `#[repr(C)]` 타입은
/// `ffi` 모듈 안에만 두고, 바깥에는 Rust의 규칙을 따르는 타입을 보입니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub low: i64,
    pub bucket_width: u32,
    pub bucket_count: u32,
}

impl Default for Config {
    fn default() -> Self {
        // safe fn으로 선언한 함수라서 unsafe 블록 없이 부릅니다.
        let c = ffi::tally_default_config();
        Config {
            low: c.low,
            bucket_width: c.bucket_width,
            bucket_count: c.bucket_count,
        }
    }
}

impl From<Config> for ffi::tally_config {
    fn from(c: Config) -> Self {
        ffi::tally_config {
            low: c.low,
            bucket_width: c.bucket_width,
            bucket_count: c.bucket_count,
        }
    }
}

/// 기록한 표본의 요약
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    pub count: u64,
    pub below: u64,
    pub above: u64,
    /// (최솟값, 최댓값). 표본이 없으면 `None`입니다(C는 이때 0을 넣어 둡니다).
    pub range: Option<(i64, i64)>,
}

/// 래퍼가 돌려주는 에러. C의 NULL과 상태 코드를 Rust의 `Result`로 바꿉니다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TallyError {
    /// `tally_new`가 NULL을 돌려줌(설정이 잘못됐거나 메모리가 없음)
    CreateFailed,
    /// 이름이 `NAME_MAX` 바이트를 넘음
    NameTooLong { len: usize, max: usize },
    /// 이름 중간에 0 바이트가 있어 C 문자열로 바꿀 수 없음
    InteriorNul { position: usize },
    /// `tally_render`가 NULL을 돌려줌(메모리가 없음)
    RenderFailed,
    /// 래퍼가 예상하지 않은 상태 코드
    Status(i32),
}

impl fmt::Display for TallyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TallyError::CreateFailed => write!(f, "tally를 만들 수 없습니다(설정을 확인하세요)"),
            TallyError::NameTooLong { len, max } => {
                write!(f, "이름이 {len}바이트로 최대 {max}바이트를 넘습니다")
            }
            TallyError::InteriorNul { position } => {
                write!(f, "이름의 {position}번 위치에 0 바이트가 있습니다")
            }
            TallyError::RenderFailed => write!(f, "표를 만들 메모리가 없습니다"),
            TallyError::Status(code) => match status_name(*code) {
                Some(name) => write!(f, "예상하지 않은 상태 코드 {name}({code})"),
                None => write!(f, "래퍼가 모르는 상태 코드 {code}"),
            },
        }
    }
}

impl Error for TallyError {}

/// 상태 코드에 해당하는 헤더의 이름. 래퍼가 모르는 값이 와도 정수로 받았으므로 안전합니다.
fn status_name(code: i32) -> Option<&'static str> {
    match code {
        ffi::TALLY_OK => Some("TALLY_OK"),
        ffi::TALLY_ERR_NULL => Some("TALLY_ERR_NULL"),
        ffi::TALLY_ERR_NAME_TOO_LONG => Some("TALLY_ERR_NAME_TOO_LONG"),
        _ => None,
    }
}

/// C의 `tally *`를 소유하는 래퍼입니다.
///
/// 필드가 원시 포인터(`NonNull`)라서 `Send`와 `Sync`가 자동으로 구현되지 않습니다.
/// 헤더의 스레드 규칙을 읽고 `Send`만 직접 구현했습니다(아래 `unsafe impl` 참고).
pub struct Tally {
    raw: NonNull<ffi::tally>,
}

impl Tally {
    /// 새 tally를 만듭니다. C가 NULL을 돌려주면 `Err`입니다.
    pub fn new(config: Config) -> Result<Tally, TallyError> {
        // tally_new는 safe fn입니다. C가 설정을 검사하고, 잘못되면 NULL을 돌려줍니다.
        let raw = ffi::tally_new(config.into());
        NonNull::new(raw)
            .map(|raw| Tally { raw })
            .ok_or(TallyError::CreateFailed)
    }

    /// 표본 하나를 기록합니다. C 함수가 포인터를 고치므로 `&mut self`를 받습니다.
    pub fn record(&mut self, sample: i64) {
        // SAFETY: self.raw는 tally_new가 돌려준 NULL이 아닌 포인터이고, drop 전에는 해제되지
        // 않습니다. &mut self이므로 이 호출 동안 다른 누구도 같은 tally를 쓰지 않습니다.
        let status = unsafe { ffi::tally_record(self.raw.as_ptr(), sample) };
        // NULL이 아닌 포인터에는 TALLY_OK만 돌려준다는 것이 헤더의 약속입니다.
        debug_assert_eq!(status, ffi::TALLY_OK);
    }

    /// 요약을 돌려줍니다. C가 값으로 돌려준 `#[repr(C)]` 구조체를 Rust 타입으로 옮깁니다.
    pub fn summary(&self) -> Summary {
        // SAFETY: self.raw는 살아 있는 tally를 가리키고, tally_summarize는 const 포인터로 읽기만 합니다.
        // Tally는 Sync가 아니므로 다른 스레드가 같은 tally를 동시에 읽는 일도 없습니다.
        let s = unsafe { ffi::tally_summarize(self.raw.as_ptr()) };
        Summary {
            count: s.count,
            below: s.below,
            above: s.above,
            range: (s.count > 0).then_some((s.min, s.max)),
        }
    }

    /// 이름을 바꿉니다. C가 내용을 복사해 두므로 `name`은 이 호출 동안만 살아 있으면 됩니다.
    pub fn set_name(&mut self, name: &str) -> Result<(), TallyError> {
        // 소유한 C 문자열이 필요하므로 CString을 만듭니다. 중간의 0 바이트는 여기서 거릅니다.
        let c_name = CString::new(name).map_err(|e| TallyError::InteriorNul {
            position: e.nul_position(),
        })?;
        // SAFETY: self.raw는 살아 있고 &mut self입니다. c_name은 이 문장이 끝날 때까지 살아 있는
        // 0 바이트로 끝나는 문자열이고, 헤더에 따르면 C는 내용을 복사할 뿐 포인터를 보관하지 않습니다.
        let status = unsafe { ffi::tally_set_name(self.raw.as_ptr(), c_name.as_ptr()) };
        match status {
            ffi::TALLY_OK => Ok(()),
            ffi::TALLY_ERR_NAME_TOO_LONG => Err(TallyError::NameTooLong {
                len: name.len(),
                max: ffi::TALLY_NAME_MAX,
            }),
            other => Err(TallyError::Status(other)),
        }
    }

    /// C가 빌려주는 이름을 `&CStr`로 돌려줍니다.
    ///
    /// C의 약속은 "다음 `tally_set_name`이나 `tally_free`까지 유효"입니다. 돌려주는 참조의
    /// 라이프타임을 `&self`에 묶었으므로, 참조를 쥔 채 `set_name`(`&mut self`)을 부르거나
    /// `Tally`를 버리는 코드는 대여 검사기가 거부합니다.
    ///
    /// ```compile_fail
    /// let mut t = tally::Tally::new(tally::Config::default()).unwrap();
    /// let old = t.name();
    /// t.set_name("새 이름").unwrap(); // old가 아직 쓰이므로 &mut 대여를 만들 수 없습니다
    /// println!("{old:?}");
    /// ```
    pub fn name(&self) -> &CStr {
        // SAFETY: self.raw는 살아 있는 tally를 가리킵니다.
        let ptr = unsafe { ffi::tally_name(self.raw.as_ptr()) };
        // NULL이 아닌 t에는 NULL이 아닌 포인터를 돌려준다는 것이 헤더의 약속입니다.
        debug_assert!(!ptr.is_null());
        // SAFETY: ptr은 t 안의 0 바이트로 끝나는 버퍼를 가리킵니다. 그 버퍼를 무효로 만드는
        // tally_set_name과 tally_free는 &mut self나 소유권이 있어야 부를 수 있으므로,
        // 돌려주는 &CStr이 살아 있는(= &self를 빌린) 동안 버퍼는 그대로입니다.
        unsafe { CStr::from_ptr(ptr) }
    }

    /// C가 할당한 표 문자열을 복사해 `String`으로 돌려주고, C의 해제 함수로 해제합니다.
    pub fn render(&self) -> Result<String, TallyError> {
        // SAFETY: self.raw는 살아 있고, tally_render는 const 포인터로 읽기만 합니다.
        let raw = unsafe { ffi::tally_render(self.raw.as_ptr()) };
        let owned = CAllocatedString::new(raw).ok_or(TallyError::RenderFailed)?;
        Ok(owned.as_c_str().to_string_lossy().into_owned())
        // 여기서 owned가 버려지며 tally_string_free가 불립니다.
    }
}

impl Drop for Tally {
    fn drop(&mut self) {
        // SAFETY: self.raw는 tally_new가 돌려준 포인터입니다. 해제하는 곳은 여기 한 곳뿐이고
        // drop은 한 번만 불리므로 이중 해제가 없습니다.
        unsafe { ffi::tally_free(self.raw.as_ptr()) }
    }
}

// SAFETY: 헤더는 "전역 상태가 없고, tally 하나를 다른 스레드로 넘겨 써도 된다"고 약속합니다.
// 그러나 "같은 tally를 여러 스레드에서 동시에 부르면 안 된다(읽기 함수끼리도)"고 하므로
// &Tally를 여러 스레드가 나눠 쥐게 하는 Sync는 구현하지 않습니다.
unsafe impl Send for Tally {}

/// `tally_render`가 할당한 문자열. 버려질 때 `tally_string_free`로 해제합니다.
struct CAllocatedString {
    ptr: NonNull<c_char>,
}

impl CAllocatedString {
    fn new(ptr: *mut c_char) -> Option<CAllocatedString> {
        NonNull::new(ptr).map(|ptr| CAllocatedString { ptr })
    }

    fn as_c_str(&self) -> &CStr {
        // SAFETY: ptr은 tally_render가 돌려준 NULL이 아닌, 0 바이트로 끝나는 문자열입니다.
        // 해제는 drop에서만 하므로 &self를 빌린 동안 유효합니다.
        unsafe { CStr::from_ptr(self.ptr.as_ptr()) }
    }
}

impl Drop for CAllocatedString {
    fn drop(&mut self) {
        // SAFETY: ptr은 tally_render가 할당한 문자열이고 해제는 여기서 한 번만 합니다.
        // Rust의 할당기나 C 표준 라이브러리의 free가 아니라 라이브러리가 짝지어 준 함수를 씁니다.
        unsafe { ffi::tally_string_free(self.ptr.as_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn latency() -> Tally {
        let config = Config {
            low: 0,
            bucket_width: 50,
            bucket_count: 4,
        };
        let mut t = Tally::new(config).unwrap();
        for x in [12, 48, 51, 77, 99, 130, -3, 250, 0, 49, 199, 200] {
            t.record(x);
        }
        t
    }

    #[test]
    fn default_config_comes_from_c() {
        let c = Config::default();
        assert_eq!((c.low, c.bucket_width, c.bucket_count), (0, 10, 10));
    }

    #[test]
    fn summary_counts_out_of_range() {
        let s = latency().summary();
        assert_eq!((s.count, s.below, s.above), (12, 1, 2));
        assert_eq!(s.range, Some((-3, 250)));
    }

    #[test]
    fn empty_summary_has_no_range() {
        let t = Tally::new(Config::default()).unwrap();
        assert_eq!(t.summary().range, None);
    }

    #[test]
    fn invalid_configs_become_err() {
        let base = Config::default();
        let bad = [
            Config {
                bucket_width: 0,
                ..base
            },
            Config {
                bucket_count: 0,
                ..base
            },
            Config {
                bucket_count: MAX_BUCKETS + 1,
                ..base
            },
            Config {
                low: i64::MAX - 5,
                ..base
            },
        ];
        for config in bad {
            assert_eq!(Tally::new(config).err(), Some(TallyError::CreateFailed));
        }
    }

    #[test]
    fn name_is_borrowed_from_c() {
        let mut t = latency();
        assert_eq!(t.name(), c"tally");
        t.set_name("응답 시간").unwrap();
        assert_eq!(t.name().to_str(), Ok("응답 시간"));
    }

    #[test]
    fn name_errors() {
        let mut t = latency();
        let long = "가".repeat(11); // 33바이트
        assert_eq!(
            t.set_name(&long),
            Err(TallyError::NameTooLong {
                len: 33,
                max: NAME_MAX
            })
        );
        assert_eq!(
            t.set_name("ab\0cd"),
            Err(TallyError::InteriorNul { position: 2 })
        );
        // 실패한 뒤에도 이전 이름이 그대로입니다.
        assert_eq!(t.name(), c"tally");
    }

    #[test]
    fn render_is_copied_and_freed() {
        let text = latency().render().unwrap();
        assert!(text.starts_with("[tally] n=12 below=1 above=2"));
        assert!(text.contains("  [    0,    50) #### 4\n"));
    }

    #[test]
    fn tally_can_move_to_another_thread() {
        let t = latency();
        let summary = thread::spawn(move || t.summary()).join().unwrap();
        assert_eq!(summary.count, 12);
    }
}
