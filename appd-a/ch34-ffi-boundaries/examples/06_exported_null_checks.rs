// 지침 6: C에 내보내는 함수는 이름에 접두사를 붙이고, C가 넘긴 포인터를 믿지 않습니다.
//
// 실행: cargo run -p er-ch34-ffi-boundaries --example 06_exported_null_checks
//
// 실제로는 #[unsafe(no_mangle)]을 붙여 C가 이름으로 찾게 하지만(projects/textstat_export),
// 이 예제는 같은 실행 파일 안의 Rust 코드가 C 호출자 역할을 하므로 속성 없이 둡니다.
// NULL을 &*p로 참조로 바꾸는 비권장 코드는 미정의 동작이라 NULL로는 실행하지 않습니다.

use std::ptr;

/// C 쪽이 가진 구조체: struct er34_stats { uint32_t samples; int64_t sum; };
#[repr(C)]
pub struct Stats {
    pub samples: u32,
    pub sum: i64,
}

mod v1 {
    use super::Stats;

    /// 비권장: 접두사 없는 이름, 받은 포인터를 바로 참조로 바꿈, 0으로 나누기를 그대로 둠
    ///
    /// # Safety
    /// `stats`는 초기화된 Stats를 가리켜야 합니다(NULL이면 미정의 동작).
    pub unsafe extern "C" fn mean(stats: *const Stats) -> f64 {
        // SAFETY: 이 함수의 전제 조건에 기댑니다. C 쪽에서 NULL이 오면 그대로 미정의 동작입니다.
        let s = unsafe { &*stats };
        s.sum as f64 / f64::from(s.samples)
    }

    /// 비권장: 지역 변수의 주소를 돌려줌. 함수가 끝나면 그 스택 메모리는 다른 용도로 쓰입니다.
    #[expect(
        dangling_pointers_from_locals,
        reason = "지역 변수의 주소를 돌려주면 rustc가 경고한다는 것을 보입니다"
    )]
    pub extern "C" fn default_stats() -> *const Stats {
        let local = Stats { samples: 0, sum: 0 };
        &raw const local
    }
}

mod v2 {
    use super::Stats;

    pub const ER34_OK: i32 = 0;
    pub const ER34_ERR_NULL: i32 = -1;
    pub const ER34_ERR_EMPTY: i32 = -2;

    /// 권장: 접두사, NULL 검사, 결과는 출력 매개변수로, 실패는 상태 코드로
    ///
    /// # Safety
    /// `stats`는 NULL이거나 초기화된 Stats를 가리켜야 하고, `out`은 NULL이거나 f64 하나를 쓸 수 있는
    /// 정렬된 메모리를 가리켜야 합니다(초기화돼 있지 않아도 됩니다).
    pub unsafe extern "C" fn er34_stats_mean(stats: *const Stats, out: *mut f64) -> i32 {
        // SAFETY: stats는 NULL이거나 초기화된 Stats를 가리킨다는 것이 이 함수의 전제 조건입니다.
        // 읽을 값이 있는 입력 포인터이므로 as_ref()로 참조를 만듭니다(NULL은 None).
        let Some(s) = (unsafe { stats.as_ref() }) else {
            return ER34_ERR_NULL;
        };
        // out은 쓰기만 하는 출력 매개변수입니다. C 호출자는 초기화하지 않은 변수의 주소를 넘기기도
        // 하므로 as_mut()으로 &mut f64를 만들지 않고, NULL만 확인한 뒤 원시 포인터로 씁니다.
        if out.is_null() {
            return ER34_ERR_NULL;
        }
        if s.samples == 0 {
            return ER34_ERR_EMPTY;
        }
        // SAFETY: out이 NULL이 아님을 위에서 확인했고, f64 하나를 쓸 수 있는 정렬된 메모리라는 것이
        // 전제 조건입니다. write는 원래 내용을 읽지 않으므로 초기화돼 있지 않아도 됩니다.
        unsafe { out.write(s.sum as f64 / f64::from(s.samples)) };
        ER34_OK
    }

    /// 권장: 스택 대신 프로그램이 끝날 때까지 사는 정적 데이터의 주소를 돌려줍니다.
    pub extern "C" fn er34_stats_default() -> *const Stats {
        static DEFAULT: Stats = Stats { samples: 0, sum: 0 };
        &raw const DEFAULT
    }
}

fn main() {
    let stats = Stats {
        samples: 4,
        sum: 250,
    };
    let empty = Stats { samples: 0, sum: 0 };

    println!("1. 비권장: 받은 포인터를 그대로 믿음");
    // SAFETY: &stats는 살아 있는 Stats를 가리킵니다.
    println!("   mean(&stats) = {}", unsafe { v1::mean(&stats) });
    // SAFETY: &empty는 살아 있는 Stats를 가리킵니다.
    println!("   mean(&empty) = {}", unsafe { v1::mean(&empty) });
    // 출력:    mean(&stats) = 62.5
    // 출력:    mean(&empty) = NaN
    // 0/0이 NaN으로 조용히 나갑니다. NULL을 넘기면 미정의 동작이므로 실행하지 않습니다.

    println!("2. 권장: NULL과 빈 입력을 상태 코드로");
    let mut out = 0.0;
    let out_ptr: *mut f64 = &mut out;
    let cases: [(&str, *const Stats, *mut f64); 4] = [
        ("&stats, &out", &stats, out_ptr),
        ("NULL, &out", ptr::null(), out_ptr),
        ("&stats, NULL", &stats, ptr::null_mut()),
        ("&empty, &out", &empty, out_ptr),
    ];
    for (args, s, o) in cases {
        // SAFETY: s와 o는 NULL이거나 main의 지역 값(stats, empty, out)을 가리키고,
        // 그 값들은 반복문이 끝날 때까지 살아 있습니다.
        let status = unsafe { v2::er34_stats_mean(s, o) };
        println!("   er34_stats_mean({args}) = {status}");
    }
    println!("   out = {out}");
    // 출력:    er34_stats_mean(&stats, &out) = 0
    // 출력:    er34_stats_mean(NULL, &out) = -1
    // 출력:    er34_stats_mean(&stats, NULL) = -1
    // 출력:    er34_stats_mean(&empty, &out) = -2
    // 출력:    out = 62.5
    println!(
        "   (ER34_OK = {}, ER34_ERR_NULL = {}, ER34_ERR_EMPTY = {})",
        v2::ER34_OK,
        v2::ER34_ERR_NULL,
        v2::ER34_ERR_EMPTY
    );
    // 출력:    (ER34_OK = 0, ER34_ERR_NULL = -1, ER34_ERR_EMPTY = -2)

    println!("3. 돌려주는 포인터가 가리키는 곳");
    // v1의 포인터는 이미 끝난 스택 프레임을 가리킵니다. NULL인지만 보고 읽지 않습니다.
    println!(
        "   v1::default_stats()가 NULL인가: {}",
        v1::default_stats().is_null()
    );
    // SAFETY: er34_stats_default는 정적 변수의 주소를 돌려주므로 언제나 읽을 수 있습니다.
    let d = unsafe { &*v2::er34_stats_default() };
    println!("   v2::er34_stats_default().samples = {}", d.samples);
    // 출력:    v1::default_stats()가 NULL인가: false
    // 출력:    v2::er34_stats_default().samples = 0
}
