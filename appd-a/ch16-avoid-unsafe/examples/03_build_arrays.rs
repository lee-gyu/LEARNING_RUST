//! 지침 1: `unsafe`를 쓰기 전에 표준 라이브러리에서 찾는다 — 원소가 `Copy`가 아닌 배열을 만들 때.
//!
//! 우체국 분류기에는 우편번호 첫 자리(0~9)마다 칸이 하나씩 있습니다. 칸(`Bin`)은 이름표와
//! 우편물 목록을 가지므로 `Copy`가 아니고, `[칸; 10]` 꼴의 배열 식은 E0277로 거부됩니다
//! (compile_fail/e0277_array_repeat_non_copy.rs). 여기서 `MaybeUninit`으로 칸을 하나씩 채우는
//! 판(비권장)과 `std::array::from_fn` 등으로 만드는 판(권장)을 비교합니다.
//!
//! 2번 구역은 칸을 만들다 중간에 실패하는 경우입니다. 비권장 판은 그때까지 만든 칸을 아무도
//! 해제하지 않아 메모리가 샙니다. 누수는 미정의 동작이 아니므로 안전한 실험이며, 살아 있는 칸의
//! 수를 세어 눈으로 확인합니다.
//! 이 파일의 `unsafe` 블록은 Miri로 검사하지 못했습니다(nightly 툴체인을 설치하지 않음).
//!
//! 실행: cargo run -p er-ch16-avoid-unsafe --example 03_build_arrays

use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

/// 지금 살아 있는(아직 drop되지 않은) 칸의 수.
static LIVE_BINS: AtomicUsize = AtomicUsize::new(0);

const BINS: usize = 10;

struct Bin {
    label: String,
    carrier: String,
    letters: Vec<&'static str>,
}

impl Bin {
    /// 담당 집배원이 아직 정해지지 않은 칸.
    fn new(digit: usize) -> Bin {
        Bin::build(digit, "미정")
    }

    /// 담당 집배원 이름이 비어 있으면 실패합니다.
    fn with_carrier(digit: usize, carrier: &str) -> Result<Bin, String> {
        if carrier.is_empty() {
            return Err(format!("{digit}번 칸의 담당자 이름이 비어 있음"));
        }
        Ok(Bin::build(digit, carrier))
    }

    fn build(digit: usize, carrier: &str) -> Bin {
        LIVE_BINS.fetch_add(1, Ordering::Relaxed);
        Bin {
            label: format!("{digit}구역"),
            carrier: carrier.to_string(),
            letters: Vec::new(),
        }
    }
}

impl Drop for Bin {
    fn drop(&mut self) {
        LIVE_BINS.fetch_sub(1, Ordering::Relaxed);
    }
}

/// 비권장: MaybeUninit 문서의 "배열을 원소 하나씩 초기화하기"를 따라 쓴 판.
mod uninit {
    use super::{BINS, Bin, MaybeUninit};

    pub fn make_bins() -> [Bin; BINS] {
        let mut bins: [MaybeUninit<Bin>; BINS] = [const { MaybeUninit::uninit() }; BINS];
        for (digit, slot) in bins.iter_mut().enumerate() {
            slot.write(Bin::new(digit));
        }
        // SAFETY: 위 반복문이 BINS칸 모두에 write를 한 번씩 했으므로 모든 칸이 초기화돼 있습니다.
        unsafe { MaybeUninit::<[Bin; BINS]>::from(bins).assume_init() }
    }

    pub fn bins_with_carriers(carriers: &[&str; BINS]) -> Result<[Bin; BINS], String> {
        let mut bins: [MaybeUninit<Bin>; BINS] = [const { MaybeUninit::uninit() }; BINS];
        for (digit, (slot, name)) in bins.iter_mut().zip(carriers).enumerate() {
            // 실패하면 ?가 여기서 반환합니다. MaybeUninit은 drop될 때 안의 값을 해제하지 않으므로
            // 이미 채운 칸은 새어 나갑니다(미정의 동작은 아님). 막으려면 채운 칸까지만 골라
            // assume_init_drop을 부르는 unsafe 코드를 더 써야 합니다.
            slot.write(Bin::with_carrier(digit, name)?);
        }
        // SAFETY: 반복문이 끝까지 돌았다면 BINS칸 모두에 write를 했습니다. 중간에 실패하면 위에서 반환합니다.
        Ok(unsafe { MaybeUninit::<[Bin; BINS]>::from(bins).assume_init() })
    }
}

/// 권장: 표준 라이브러리가 초기화와 실패 시 정리를 맡습니다.
mod safe {
    use super::{BINS, Bin};

    pub fn make_bins() -> [Bin; BINS] {
        // from_fn은 0..BINS의 인덱스마다 클로저를 불러 배열을 채웁니다.
        std::array::from_fn(Bin::new)
    }

    pub fn bins_with_carriers(carriers: &[&str; BINS]) -> Result<[Bin; BINS], String> {
        // 실패할 수 있는 초기화는 Vec으로 모은 뒤 배열로 바꿉니다. 실패하면 이미 만든 칸은
        // Vec과 함께 drop됩니다. (실패할 수 있는 array::try_from_fn은 1.100.0에서 nightly 전용입니다.)
        let bins: Vec<Bin> = carriers
            .iter()
            .enumerate()
            .map(|(digit, name)| Bin::with_carrier(digit, name))
            .collect::<Result<_, _>>()?;
        bins.try_into()
            .map_err(|v: Vec<Bin>| format!("칸이 {}개뿐", v.len()))
    }
}

/// (우편번호, 받는 사람)
const LETTERS: [(&str, &str); 6] = [
    ("06236", "김하늘"),
    ("13529", "이도윤"),
    ("48058", "박서연"),
    ("63309", "최유진"),
    ("04524", "정민준"),
    ("34126", "한지우"),
];

fn sort_letters(bins: &mut [Bin; BINS]) {
    for (code, name) in LETTERS {
        // 첫 글자가 숫자가 아니면 건너뜁니다. 인덱싱 대신 get_mut을 써서 범위 밖도 None으로 받습니다.
        let digit = code.chars().next().and_then(|c| c.to_digit(10));
        if let Some(bin) = digit.and_then(|d| bins.get_mut(d as usize)) {
            bin.letters.push(name);
        }
    }
}

fn labels(bins: &[Bin]) -> String {
    let (Some(first), Some(last)) = (bins.first(), bins.last()) else {
        return String::from("(칸 없음)");
    };
    format!(
        "{} … {} ({}칸, 담당 {})",
        first.label,
        last.label,
        bins.len(),
        first.carrier
    )
}

fn leaked_after(f: impl FnOnce() -> Result<[Bin; BINS], String>) -> (Result<(), String>, usize) {
    let before = LIVE_BINS.load(Ordering::Relaxed);
    let result = f().map(drop);
    let after = LIVE_BINS.load(Ordering::Relaxed);
    (result, after - before)
}

fn main() {
    println!("1. 칸 {BINS}개 만들기");
    let unsafe_bins = uninit::make_bins();
    let mut safe_bins = safe::make_bins();
    println!("    비권장 MaybeUninit:  {}", labels(&unsafe_bins));
    // 출력:     비권장 MaybeUninit:  0구역 … 9구역 (10칸, 담당 미정)
    println!("    권장 array::from_fn: {}", labels(&safe_bins));
    // 출력:     권장 array::from_fn: 0구역 … 9구역 (10칸, 담당 미정)
    drop(unsafe_bins);

    sort_letters(&mut safe_bins);
    let summary: Vec<String> = safe_bins
        .iter()
        .filter(|b| !b.letters.is_empty())
        .map(|b| format!("{} {}통", b.label, b.letters.len()))
        .collect();
    println!(
        "    우편물 {}통 분류: {}",
        LETTERS.len(),
        summary.join(", ")
    );
    // 출력:     우편물 6통 분류: 0구역 2통, 1구역 1통, 3구역 1통, 4구역 1통, 6구역 1통
    drop(safe_bins);
    println!("    살아 있는 칸: {}개", LIVE_BINS.load(Ordering::Relaxed)); // 출력:     살아 있는 칸: 0개

    println!("2. 담당자 목록으로 칸 만들기 (4번 칸 담당자가 비어 있음)");
    let carriers = [
        "가람", "나래", "다솜", "라온", "", "바다", "사랑", "아라", "자람", "하늘",
    ];
    let (result, leaked) = leaked_after(|| uninit::bins_with_carriers(&carriers));
    println!("    비권장 MaybeUninit + ?: {result:?}, 새어 나간 칸 {leaked}개");
    // 출력:     비권장 MaybeUninit + ?: Err("4번 칸의 담당자 이름이 비어 있음"), 새어 나간 칸 4개
    let (result, leaked) = leaked_after(|| safe::bins_with_carriers(&carriers));
    println!("    권장 collect + try_into: {result:?}, 새어 나간 칸 {leaked}개");
    // 출력:     권장 collect + try_into: Err("4번 칸의 담당자 이름이 비어 있음"), 새어 나간 칸 0개

    let carriers_ok = carriers.map(|c| if c.is_empty() { "마루" } else { c });
    let (result, leaked) = leaked_after(|| safe::bins_with_carriers(&carriers_ok));
    println!("    빈 이름을 채운 뒤 권장 판: {result:?}, 새어 나간 칸 {leaked}개");
    // 출력:     빈 이름을 채운 뒤 권장 판: Ok(()), 새어 나간 칸 0개
}
