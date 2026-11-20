// 지침: 내 코드가 기대는 의존성의 동작은, 그 동작만 확인하는 작은 테스트로 적어 둡니다(조기 경보).
//
// 실행: cargo run -p er-ch30-beyond-unit-tests --example 02_early_warning_dependency
// 테스트: cargo test -p er-ch30-beyond-unit-tests --example 02_early_warning_dependency
//
// 연락처 CSV를 읽는 내 코드가 의존성 `fieldsplit`의 `split`에 기댑니다. 두 버전을 모듈로 흉내 냅니다.
// 1.5.0은 시그니처가 같아 컴파일 에러가 나지 않고, `^1.4`라는 버전 요구 사항이면
// `cargo update` 한 번으로 들어옵니다(ch21). 같은 테스트 두 종류를 두 버전에 돌려 봅니다.

/// 의존성 `fieldsplit` 1.4.0: 쉼표로 나누고 앞뒤 공백을 지웁니다. 빈 칸도 남깁니다.
mod fieldsplit_1_4 {
    pub fn split(line: &str) -> Vec<&str> {
        line.split(',').map(str::trim).collect()
    }
}

/// 같은 의존성 1.5.0: 변경 기록에 "빈 칸을 정리합니다"라고 적힌 개선입니다.
mod fieldsplit_1_5 {
    pub fn split(line: &str) -> Vec<&str> {
        line.split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect()
    }
}

type SplitFn = fn(&str) -> Vec<&str>;

/// 내 코드: 칸의 **위치**로 이름, 이메일, 전화번호를 읽습니다. 빈 칸이 남는다는 가정에 기댑니다.
#[derive(Debug, PartialEq, Eq)]
struct Contact<'a> {
    name: &'a str,
    email: Option<&'a str>,
    phone: Option<&'a str>,
}

fn parse_contact(split: SplitFn, line: &str) -> Contact<'_> {
    let cols = split(line);
    let col = |i: usize| cols.get(i).copied().filter(|s| !s.is_empty());
    Contact {
        name: col(0).unwrap_or("(이름 없음)"),
        email: col(1),
        phone: col(2),
    }
}

/// 기능 테스트: 내 코드가 흔한 줄을 맞게 읽는지 봅니다.
fn feature_test(split: SplitFn) -> Result<(), String> {
    let got = parse_contact(split, "김하늘, sky@example.com, 010-1234-5678");
    let expected = Contact {
        name: "김하늘",
        email: Some("sky@example.com"),
        phone: Some("010-1234-5678"),
    };
    if got == expected {
        Ok(())
    } else {
        Err(format!("{got:?}"))
    }
}

/// 조기 경보 테스트: 내 코드가 기대는 의존성의 동작 **하나만** 확인합니다.
/// 실패 메시지가 원인(의존성의 동작 변화)을 바로 가리키도록 적습니다.
fn early_warning(split: SplitFn) -> Result<(), String> {
    let got = split("a,,b");
    if got == ["a", "", "b"] {
        Ok(())
    } else {
        Err(format!(
            "fieldsplit::split가 빈 칸을 남긴다는 가정이 깨졌습니다: {got:?}"
        ))
    }
}

fn verdict(r: Result<(), String>) -> String {
    match r {
        Ok(()) => "통과".to_string(),
        Err(why) => format!("실패 — {why}"),
    }
}

fn main() {
    let versions: [(&str, SplitFn); 2] = [
        ("fieldsplit 1.4.0", fieldsplit_1_4::split),
        ("fieldsplit 1.5.0", fieldsplit_1_5::split),
    ];
    for (name, split) in versions {
        println!("[{name}]");
        println!("  기능 테스트: {}", verdict(feature_test(split)));
        println!("  조기 경보 테스트: {}", verdict(early_warning(split)));
        // 이메일 칸이 빈 줄: 실제 데이터에는 있지만 기능 테스트에는 없던 경우
        let c = parse_contact(split, "박서준, , 010-9876-5432");
        println!("  이메일이 빈 줄: email={:?}, phone={:?}", c.email, c.phone);
    }
    // 출력:
    // [fieldsplit 1.4.0]
    //   기능 테스트: 통과
    //   조기 경보 테스트: 통과
    //   이메일이 빈 줄: email=None, phone=Some("010-9876-5432")
    // [fieldsplit 1.5.0]
    //   기능 테스트: 통과
    //   조기 경보 테스트: 실패 — fieldsplit::split가 빈 칸을 남긴다는 가정이 깨졌습니다: ["a", "b"]
    //   이메일이 빈 줄: email=Some("010-9876-5432"), phone=None
}

// 실제 프로젝트에서는 지금 쓰는 버전(1.4)에 대해 두 테스트를 모두 둡니다.
// 1.5로 올리는 `cargo update`가 CI에서 조기 경보 테스트를 깨뜨려 원인을 알려 줍니다.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_common_line() {
        assert_eq!(feature_test(fieldsplit_1_4::split), Ok(()));
    }

    /// 의존성에 대한 가정: `split`은 빈 칸을 지우지 않는다.
    #[test]
    fn dependency_keeps_empty_fields() {
        assert_eq!(fieldsplit_1_4::split("a,,b"), ["a", "", "b"]);
    }
}
