//! C 프로그램이 부르는 Rust 함수들입니다. C 쪽 선언은 `csrc/textstat.h`에 손으로 적었습니다.
//!
//! 이 모듈이 지키는 규칙
//! - 내보내는 함수는 `#[unsafe(no_mangle)]`과 `textstat_` 접두사를 붙입니다.
//! - 포인터를 받는 함수는 `unsafe extern "C" fn`으로 두고 `# Safety`에 조건을 적습니다.
//!   NULL은 먼저 확인해 에러 코드로 돌려줍니다. 읽을 값이 있는 포인터(핸들, 입력 구조체)는
//!   `as_ref()`/`as_mut()`으로 참조로 바꾸고, 결과를 쓰기만 하는 출력 매개변수는 초기화돼 있지
//!   않을 수 있으므로 참조로 바꾸지 않고 `is_null()` 확인 뒤 `write`로 씁니다.
//! - Rust가 할당한 `Counter`는 Rust가 해제합니다(`textstat_counter_new`/`textstat_counter_free`).
//! - 문자열은 C가 준 버퍼에 써 줍니다(할당과 해제 모두 C 쪽).
//! - 패닉은 `catch_unwind`로 붙잡아 `TEXTSTAT_ERR_PANIC`으로 바꿉니다.
//!
//! unsafe 코드는 Miri로 확인하지 못했습니다. 블록마다 `// SAFETY:`로 근거를 적었습니다.

use std::ffi::{CStr, c_char};
use std::mem;
use std::panic::{self, UnwindSafe};
use std::ptr;

pub const TEXTSTAT_OK: i32 = 0;
pub const TEXTSTAT_ERR_NULL: i32 = -1;
pub const TEXTSTAT_ERR_UTF8: i32 = -2;
pub const TEXTSTAT_ERR_PANIC: i32 = -99;

/// C의 `textstat_counts`입니다. 타입 이름은 경계를 넘지 않으므로(심볼이 없음) Rust 이름을
/// 씁니다. 경계를 넘는 것은 필드의 배치뿐이고, 그것을 `#[repr(C)]`가 C와 같게 정합니다.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub bytes: u64,
    pub chars: u64,
    pub words: u64,
    pub lines: u64,
}

/// 조각으로 나뉘어 들어오는 UTF-8 텍스트를 세는 Rust 타입입니다.
/// C에는 불투명 포인터(`textstat_counter *`)로만 보이므로 `#[repr(C)]`가 필요 없습니다.
#[derive(Debug, Default)]
pub struct Counter {
    counts: Counts,
    /// 조각 끝에서 잘린 글자의 앞 바이트
    pending: Vec<u8>,
    in_word: bool,
    last: Option<char>,
    finished: bool,
}

/// 올바른 UTF-8이 아님
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidUtf8;

impl Counter {
    pub fn new() -> Counter {
        Counter::default()
    }

    /// 조각 하나를 셉니다. 글자가 조각 경계에서 잘렸으면 다음 조각을 기다립니다.
    ///
    /// # Panics
    ///
    /// `finish` 뒤에 부르면 패닉합니다. 부르는 쪽의 사용 규칙 위반이기 때문입니다.
    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), InvalidUtf8> {
        assert!(!self.finished, "finish 뒤에는 feed할 수 없습니다");
        let mut buf = mem::take(&mut self.pending);
        buf.extend_from_slice(bytes);
        match std::str::from_utf8(&buf) {
            Ok(text) => {
                self.count(text);
                Ok(())
            }
            // error_len()이 None이면 잘못된 바이트가 아니라 끝에서 글자가 잘린 것입니다.
            Err(e) if e.error_len().is_none() => {
                let (done, rest) = buf.split_at(e.valid_up_to());
                let text = std::str::from_utf8(done).map_err(|_| InvalidUtf8)?;
                self.count(text);
                self.pending = rest.to_vec();
                Ok(())
            }
            Err(_) => Err(InvalidUtf8),
        }
    }

    fn count(&mut self, text: &str) {
        self.counts.bytes += text.len() as u64;
        for ch in text.chars() {
            self.counts.chars += 1;
            if ch == '\n' {
                self.counts.lines += 1;
            }
            if ch.is_whitespace() {
                self.in_word = false;
            } else if !self.in_word {
                self.in_word = true;
                self.counts.words += 1;
            }
            self.last = Some(ch);
        }
    }

    /// 세기를 끝내고 결과를 돌려줍니다. 줄바꿈으로 끝나지 않은 마지막 줄도 한 줄로 셉니다.
    ///
    /// # Panics
    ///
    /// 두 번 부르면 패닉합니다.
    pub fn finish(&mut self) -> Result<Counts, InvalidUtf8> {
        assert!(!self.finished, "finish는 한 번만 부를 수 있습니다");
        if !self.pending.is_empty() {
            return Err(InvalidUtf8); // 마지막 글자가 잘린 채 끝남
        }
        self.finished = true;
        let mut counts = self.counts;
        if self.last.is_some_and(|c| c != '\n') {
            counts.lines += 1;
        }
        Ok(counts)
    }
}

/// 경계 안에서 패닉을 붙잡아 에러 코드로 바꿉니다. 내보내는 함수의 본문은 모두 이 안에서 돕니다.
///
/// 붙잡지 않아도 `extern "C"` 함수 밖으로 나가려는 패닉은 프로세스를 중단시킵니다(1.81부터).
/// 여기서 붙잡는 것은 C 프로그램을 끝내는 대신 에러 코드를 돌려주기 위해서입니다.
fn guard(body: impl FnOnce() -> i32 + UnwindSafe) -> i32 {
    panic::catch_unwind(body).unwrap_or(TEXTSTAT_ERR_PANIC)
}

/// 라이브러리 버전. C 쪽이 이 이름으로 찾으므로 `no_mangle`이 빠지면 링크 에러입니다.
#[unsafe(no_mangle)]
pub extern "C" fn textstat_version() -> u32 {
    1
}

/// `text` 전체를 셉니다.
///
/// # Safety
///
/// `text`는 NULL이거나 0 바이트로 끝나는 읽을 수 있는 문자열이어야 하고, `out`은 NULL이거나
/// `textstat_counts` 하나를 쓸 수 있는 정렬된 메모리를 가리켜야 합니다(초기화돼 있지 않아도 됩니다).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn textstat_count(text: *const c_char, out: *mut Counts) -> i32 {
    guard(|| {
        // out은 결과를 쓰기만 하는 출력 매개변수입니다. C 호출자는 초기화하지 않은 변수의 주소를
        // 넘기기도 하므로 as_mut()으로 참조를 만들지 않고, NULL만 확인한 뒤 아래에서 write로 씁니다.
        if text.is_null() || out.is_null() {
            return TEXTSTAT_ERR_NULL;
        }
        // SAFETY: text가 NULL이 아님을 위에서 확인했고, 0 바이트로 끝나는 문자열이라는 것이
        // 전제 조건입니다. 만든 &CStr은 이 클로저 안에서만 쓰고 보관하지 않습니다.
        let text = unsafe { CStr::from_ptr(text) };
        let mut counter = Counter::new();
        match counter
            .feed(text.to_bytes())
            .and_then(|()| counter.finish())
        {
            Ok(counts) => {
                // SAFETY: out이 NULL이 아님을 위에서 확인했고, Counts 하나를 쓸 수 있는 정렬된
                // 메모리라는 것이 전제 조건입니다. write는 원래 내용을 읽거나 버리지 않습니다.
                unsafe { out.write(counts) };
                TEXTSTAT_OK
            }
            Err(InvalidUtf8) => TEXTSTAT_ERR_UTF8,
        }
    })
}

/// 새 `Counter`를 만들어 소유권을 C에 넘깁니다. 다 쓰면 `textstat_counter_free`로 돌려줘야 합니다.
#[unsafe(no_mangle)]
pub extern "C" fn textstat_counter_new() -> *mut Counter {
    // Box를 그대로 돌려주면 이 함수 끝에서 해제됩니다. Box::into_raw는 소유권을 원시 포인터로
    // 바꿔 Rust의 자동 해제에서 빼냅니다. 이 안에서는 패닉할 일이 없어 guard를 쓰지 않습니다
    // (메모리가 없으면 패닉이 아니라 중단입니다).
    Box::into_raw(Box::new(Counter::new()))
}

/// 조각 하나를 넣습니다.
///
/// # Safety
///
/// `counter`는 NULL이거나 `textstat_counter_new`가 돌려주고 아직 해제하지 않은 포인터여야 하고,
/// 이 호출 동안 다른 스레드가 같은 핸들을 쓰면 안 됩니다. `chunk`는 NULL이거나 0 바이트로 끝나는
/// 문자열이어야 합니다.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn textstat_counter_feed(counter: *mut Counter, chunk: *const c_char) -> i32 {
    guard(|| {
        if chunk.is_null() {
            return TEXTSTAT_ERR_NULL;
        }
        // SAFETY: counter는 NULL이거나 살아 있는 Counter를 가리키고 동시에 쓰는 곳이 없다는 것이
        // 전제 조건입니다. NULL은 as_mut()이 None으로 바꿉니다.
        let Some(counter) = (unsafe { counter.as_mut() }) else {
            return TEXTSTAT_ERR_NULL;
        };
        // SAFETY: chunk가 NULL이 아님을 확인했고, 0 바이트로 끝나는 문자열이라는 것이 전제 조건입니다.
        let chunk = unsafe { CStr::from_ptr(chunk) };
        // finish 뒤라면 feed가 패닉하고, guard가 TEXTSTAT_ERR_PANIC으로 바꿉니다.
        // feed는 상태를 바꾸기 전에 검사하므로, 붙잡힌 뒤에도 Counter는 어긋나지 않습니다.
        match counter.feed(chunk.to_bytes()) {
            Ok(()) => TEXTSTAT_OK,
            Err(InvalidUtf8) => TEXTSTAT_ERR_UTF8,
        }
    })
}

/// 세기를 끝내고 결과를 `out`에 씁니다.
///
/// # Safety
///
/// `counter`는 `textstat_counter_feed`와 같은 조건을 지켜야 하고, `out`은 NULL이거나
/// `textstat_counts` 하나를 쓸 수 있는 정렬된 메모리를 가리켜야 합니다(초기화돼 있지 않아도 됩니다).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn textstat_counter_finish(counter: *mut Counter, out: *mut Counts) -> i32 {
    guard(|| {
        // out은 출력 매개변수라서 참조로 바꾸지 않고 NULL만 확인합니다. finish를 부르기 전에
        // 확인해야, 결과를 쓸 곳이 없는데 핸들만 끝난 상태가 되는 일이 없습니다.
        if out.is_null() {
            return TEXTSTAT_ERR_NULL;
        }
        // SAFETY: counter는 NULL이거나 textstat_counter_new가 만든 살아 있는(초기화된) Counter이고
        // 동시에 쓰는 곳이 없다는 것이 전제 조건입니다. NULL은 as_mut()이 None으로 바꿉니다.
        let Some(counter) = (unsafe { counter.as_mut() }) else {
            return TEXTSTAT_ERR_NULL;
        };
        match counter.finish() {
            Ok(counts) => {
                // SAFETY: out이 NULL이 아님을 위에서 확인했고, Counts 하나를 쓸 수 있는 정렬된
                // 메모리라는 것이 전제 조건입니다(초기화돼 있지 않아도 write는 괜찮습니다).
                unsafe { out.write(counts) };
                TEXTSTAT_OK
            }
            Err(InvalidUtf8) => TEXTSTAT_ERR_UTF8,
        }
    })
}

/// `textstat_counter_new`가 만든 핸들을 해제합니다. NULL이면 아무 일도 하지 않습니다.
///
/// # Safety
///
/// `counter`는 NULL이거나 `textstat_counter_new`가 돌려주고 아직 해제하지 않은 포인터여야 합니다.
/// 해제한 뒤에는 그 포인터를 쓰면 안 됩니다.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn textstat_counter_free(counter: *mut Counter) {
    if counter.is_null() {
        return;
    }
    // SAFETY: counter는 textstat_counter_new가 Box::into_raw로 만들었고 아직 해제되지 않았다는 것이
    // 전제 조건입니다. Box로 되돌리면 이 문장에서 Rust의 할당기로 해제됩니다(할당한 쪽이 해제).
    // Counter의 drop은 패닉하지 않으므로 guard를 쓰지 않습니다.
    drop(unsafe { Box::from_raw(counter) });
}

/// `counts`를 문장으로 만들어 C가 준 버퍼에 씁니다. 필요한 길이(0 바이트 제외)를 돌려줍니다.
///
/// # Safety
///
/// `counts`는 NULL이거나 초기화된 `textstat_counts`를 가리켜야 하고(읽어서 참조로 바꾸므로),
/// `buf`는 NULL이거나 `buf_len`바이트를 쓸 수 있는 버퍼여야 합니다(초기화돼 있지 않아도 됩니다).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn textstat_describe(
    counts: *const Counts,
    buf: *mut c_char,
    buf_len: usize,
) -> i32 {
    guard(|| {
        // SAFETY: counts는 NULL이거나 초기화된 Counts를 가리킨다는 것이 전제 조건입니다.
        // 읽을 값이 있는 입력 포인터이므로 as_ref()로 참조를 만들어도 됩니다(NULL은 None).
        let Some(c) = (unsafe { counts.as_ref() }) else {
            return TEXTSTAT_ERR_NULL;
        };
        let text = format!(
            "{}바이트, {}글자, {}단어, {}줄",
            c.bytes, c.chars, c.words, c.lines
        );
        let Ok(needed) = i32::try_from(text.len()) else {
            return TEXTSTAT_ERR_PANIC; // 일어나지 않지만 패닉 대신 에러 코드로
        };
        if buf.is_null() || buf_len == 0 {
            return needed; // snprintf(NULL, 0, ...)처럼 길이만 알려 줍니다
        }
        if text.len() < buf_len {
            // SAFETY: buf는 buf_len바이트를 쓸 수 있고 text.len() + 1 <= buf_len입니다. 방금 만든
            // String과 C의 버퍼는 겹치지 않습니다. &mut [u8]을 만들지 않고 원시 포인터로 쓰므로
            // C의 버퍼가 초기화돼 있지 않아도 됩니다.
            unsafe { ptr::copy_nonoverlapping(text.as_ptr(), buf.cast::<u8>(), text.len()) };
            // SAFETY: text.len() < buf_len이므로 버퍼 안입니다.
            unsafe { buf.add(text.len()).write(0) };
        } else {
            // SAFETY: buf_len > 0이므로 첫 바이트는 버퍼 안입니다.
            unsafe { buf.write(0) };
        }
        needed
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::MaybeUninit;

    // C 프로그램이 하듯 내보낸 함수를 원시 포인터로 부릅니다.

    #[test]
    fn count_whole_text() {
        // C 호출자처럼 출력 매개변수를 초기화하지 않은 채 넘깁니다.
        let mut out = MaybeUninit::<Counts>::uninit();
        // SAFETY: 문자열은 0 바이트로 끝나는 리터럴이고, out.as_mut_ptr()은 Counts 하나를 쓸 수 있는
        // 정렬된 지역 메모리를 가리킵니다(초기화되지 않아도 된다는 것이 함수의 조건입니다).
        let status =
            unsafe { textstat_count(c"hello, 러스트\n둘째 줄".as_ptr(), out.as_mut_ptr()) };
        assert_eq!(status, TEXTSTAT_OK);
        // SAFETY: TEXTSTAT_OK이면 textstat_count가 out에 값을 썼습니다.
        let out = unsafe { out.assume_init() };
        assert_eq!(
            out,
            Counts {
                bytes: 27,
                chars: 15,
                words: 4,
                lines: 2
            }
        );
    }

    #[test]
    fn null_and_bad_utf8_become_codes() {
        let mut out = Counts::default();
        // SAFETY: NULL은 허용된 값이고 나머지 포인터는 살아 있습니다.
        unsafe {
            assert_eq!(textstat_count(ptr::null(), &mut out), TEXTSTAT_ERR_NULL);
            assert_eq!(
                textstat_count(c"abc".as_ptr(), ptr::null_mut()),
                TEXTSTAT_ERR_NULL
            );
            assert_eq!(
                textstat_count(c"\xff\xfe".as_ptr(), &mut out),
                TEXTSTAT_ERR_UTF8
            );
        }
    }

    #[test]
    fn split_character_across_chunks() {
        let k = textstat_counter_new();
        let mut out = Counts::default();
        // SAFETY: k는 방금 만든 핸들이고 이 테스트만 씁니다. 마지막에 한 번만 해제합니다.
        unsafe {
            assert_eq!(textstat_counter_feed(k, c"안".as_ptr()), TEXTSTAT_OK);
            assert_eq!(textstat_counter_feed(k, c"\xeb\x85".as_ptr()), TEXTSTAT_OK);
            assert_eq!(
                textstat_counter_feed(k, c"\x95 world\n".as_ptr()),
                TEXTSTAT_OK
            );
            assert_eq!(textstat_counter_finish(k, &mut out), TEXTSTAT_OK);
            // finish 뒤의 feed는 안에서 패닉하지만 경계 밖으로는 에러 코드만 나옵니다.
            assert_eq!(
                textstat_counter_feed(k, c"more".as_ptr()),
                TEXTSTAT_ERR_PANIC
            );
            textstat_counter_free(k);
            textstat_counter_free(ptr::null_mut());
        }
        assert_eq!((out.bytes, out.chars, out.words, out.lines), (13, 9, 2, 1));
    }

    #[test]
    fn describe_into_c_buffer() {
        let counts = Counts {
            bytes: 13,
            chars: 9,
            words: 2,
            lines: 1,
        };
        let mut buf = [0x7f as c_char; 64];
        // SAFETY: counts와 buf는 살아 있고 buf_len은 buf의 실제 길이입니다.
        let n = unsafe { textstat_describe(&counts, buf.as_mut_ptr(), buf.len()) };
        // SAFETY: textstat_describe가 길이가 충분할 때 0 바이트로 끝나는 문자열을 썼습니다.
        let written = unsafe { CStr::from_ptr(buf.as_ptr()) };
        assert_eq!(written.to_str(), Ok("13바이트, 9글자, 2단어, 1줄"));
        assert_eq!(usize::try_from(n), Ok(written.count_bytes()));

        let mut tiny = [0x7f as c_char; 8];
        // SAFETY: 위와 같음
        let n2 = unsafe { textstat_describe(&counts, tiny.as_mut_ptr(), tiny.len()) };
        assert_eq!(n2, n);
        assert_eq!(tiny[0], 0); // 모자라면 빈 문자열
    }
}
