// 지침: 타입 시스템이 이미 보장하는 것은 테스트하지 않고, 타입이 보장하지 않는 것에 테스트를 씁니다.
//
// 실행: cargo run -p er-ch30-beyond-unit-tests --example 01_what_types_guarantee
// 테스트: cargo test -p er-ch30-beyond-unit-tests --example 01_what_types_guarantee
//
// `grade`는 `u8` 점수(0~255)를 받아 0~100점만 등급으로 바꿉니다. 테스트 묶음 두 개를
// 정상 판과 버그를 일부러 넣은 두 판에 각각 돌려, 어느 묶음이 버그를 잡는지 봅니다.
// "버그를 넣었을 때 실패하지 않는 테스트는 아무것도 지키지 않는다"는 것을 보이는 장치입니다.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Grade {
    A,
    B,
    C,
    F,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OutOfRange(u8);

type GradeFn = fn(u8) -> Result<Grade, OutOfRange>;
type Suite = fn(GradeFn) -> Result<(), String>;

/// 정상 판: 0~100점만 받고, 100을 넘으면 에러입니다.
fn grade(score: u8) -> Result<Grade, OutOfRange> {
    match score {
        90..=100 => Ok(Grade::A),
        80..=89 => Ok(Grade::B),
        60..=79 => Ok(Grade::C),
        0..=59 => Ok(Grade::F),
        _ => Err(OutOfRange(score)),
    }
}

/// 버그 ①: 100을 넘는 점수도 A로 받아 줍니다. `u8`은 255까지 담으므로 이 버그는 실제로 일어납니다.
fn grade_accepts_over_100(score: u8) -> Result<Grade, OutOfRange> {
    match score {
        90.. => Ok(Grade::A),
        80..=89 => Ok(Grade::B),
        60..=79 => Ok(Grade::C),
        0..=59 => Ok(Grade::F),
    }
}

/// 버그 ②: C의 아래쪽 경계가 한 칸 밀려 60점이 F가 됩니다.
fn grade_shifted_60(score: u8) -> Result<Grade, OutOfRange> {
    match score {
        90..=100 => Ok(Grade::A),
        80..=89 => Ok(Grade::B),
        61..=79 => Ok(Grade::C),
        0..=60 => Ok(Grade::F),
        _ => Err(OutOfRange(score)),
    }
}

/// 비권장 묶음: 타입이 이미 보장하는 것을 확인합니다. 어떤 구현을 넣어도 통과합니다.
fn suite_type_guarantees(f: GradeFn) -> Result<(), String> {
    for score in 0..=u8::MAX {
        // "점수가 음수가 아닌가" — u8은 음수를 담을 수 없습니다. rustc와 clippy가 이 비교를 알아봅니다.
        #[expect(
            unused_comparisons,
            clippy::absurd_extreme_comparisons,
            reason = "u8이 보장하는 것을 확인하는 비권장 코드"
        )]
        let non_negative = score >= 0;
        // "결과가 네 등급 중 하나인가" — 열거형이라 다른 값은 만들 수 없습니다.
        let known_grade = match f(score) {
            Ok(g) => matches!(g, Grade::A | Grade::B | Grade::C | Grade::F),
            Err(_) => true,
        };
        if !(non_negative && known_grade) {
            return Err(format!("{score}점"));
        }
    }
    Ok(())
}

/// 권장 묶음: 타입이 보장하지 않는 것(등급 경계, 101~255점)을 확인합니다.
fn suite_boundaries(f: GradeFn) -> Result<(), String> {
    use Grade::*;
    let cases = [
        (0, Ok(F)),
        (59, Ok(F)),
        (60, Ok(C)),
        (79, Ok(C)),
        (80, Ok(B)),
        (89, Ok(B)),
        (90, Ok(A)),
        (100, Ok(A)),
        (101, Err(OutOfRange(101))),
        (255, Err(OutOfRange(255))),
    ];
    for (score, expected) in cases {
        let got = f(score);
        if got != expected {
            return Err(format!("{score}점 → {got:?} (기대: {expected:?})"));
        }
    }
    Ok(())
}

fn verdict(r: Result<(), String>) -> String {
    match r {
        Ok(()) => "통과".to_string(),
        Err(why) => format!("실패: {why}"),
    }
}

fn main() {
    let impls: [(&str, GradeFn); 3] = [
        ("정상 판", grade),
        ("버그 ① 100점 초과를 받음", grade_accepts_over_100),
        ("버그 ② 60점 경계가 밀림", grade_shifted_60),
    ];
    let suites: [(&str, Suite); 2] = [
        ("비권장 묶음(타입이 보장하는 것)", suite_type_guarantees),
        ("권장 묶음(경계와 범위 밖)", suite_boundaries),
    ];

    for (suite_name, suite) in suites {
        println!("{suite_name}");
        for (impl_name, f) in impls {
            println!("  {impl_name}: {}", verdict(suite(f)));
        }
    }
    // 출력:
    // 비권장 묶음(타입이 보장하는 것)
    //   정상 판: 통과
    //   버그 ① 100점 초과를 받음: 통과
    //   버그 ② 60점 경계가 밀림: 통과
    // 권장 묶음(경계와 범위 밖)
    //   정상 판: 통과
    //   버그 ① 100점 초과를 받음: 실패: 101점 → Ok(A) (기대: Err(OutOfRange(101)))
    //   버그 ② 60점 경계가 밀림: 실패: 60점 → Ok(F) (기대: Ok(C))
}

// 실제 프로젝트에서는 권장 묶음만 단위 테스트로 남깁니다.
// "음수 점수를 거부하는가"라는 테스트는 쓸 수조차 없습니다(compile_fail/e0600_negative_score_test.rs).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_and_out_of_range() {
        assert_eq!(suite_boundaries(grade), Ok(()));
    }

    #[test]
    fn boundary_suite_catches_both_bugs() {
        // 테스트가 버그를 잡는지도 확인할 수 있습니다. 버그를 넣은 판에서는 실패해야 합니다.
        assert!(suite_boundaries(grade_accepts_over_100).is_err());
        assert!(suite_boundaries(grade_shifted_60).is_err());
    }
}
