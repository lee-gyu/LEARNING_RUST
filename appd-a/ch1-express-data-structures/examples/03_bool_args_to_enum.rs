// 지침: 뜻이 서로 다른 bool 매개변수 여러 개 대신, 선택지마다 열거형을 하나씩 만듭니다.
// 비권장: count_flags(text, word, case_sensitive: bool, whole_word: bool)
// 권장:   count(text, word, Case, Scope)
//
// 실행: cargo run -p er-ch1-express-data-structures --example 03_bool_args_to_enum

const TEXT: &str = "Rust rust RUSTY trust Rust";
const WORD: &str = "rust";

// ---- 비권장 ----------------------------------------------------------------

// 두 bool은 타입이 같으므로, 호출하는 쪽이 순서를 바꿔 넘겨도 컴파일러는 알 수 없습니다.
// 호출 지점의 `true, false`만 보고는 무엇을 켰는지도 알 수 없습니다.
fn count_flags(text: &str, word: &str, case_sensitive: bool, whole_word: bool) -> usize {
    let (text, word) = if case_sensitive {
        (text.to_string(), word.to_string())
    } else {
        (text.to_lowercase(), word.to_lowercase())
    };
    if whole_word {
        text.split_whitespace().filter(|w| *w == word).count()
    } else {
        text.matches(word.as_str()).count()
    }
}

// ---- 권장 ------------------------------------------------------------------

// 선택지마다 따로 타입을 만듭니다. Case와 Scope는 서로 다른 타입이므로
// 자리를 바꿔 넘기면 E0308로 컴파일되지 않습니다(compile_fail/e0308_swapped_enum_args.rs).
#[derive(Debug, Clone, Copy)]
enum Case {
    Sensitive,
    Insensitive,
}

#[derive(Debug, Clone, Copy)]
enum Scope {
    Substring,
    WholeWord,
}

impl Case {
    // 배리언트를 모두 적은 match입니다. 나중에 배리언트가 늘면 이곳이 E0004로 멈춰 섭니다.
    fn label(self) -> &'static str {
        match self {
            Case::Sensitive => "대소문자 구분",
            Case::Insensitive => "대소문자 무시",
        }
    }

    fn normalize(self, s: &str) -> String {
        match self {
            Case::Sensitive => s.to_string(),
            Case::Insensitive => s.to_lowercase(),
        }
    }
}

impl Scope {
    fn label(self) -> &'static str {
        match self {
            Scope::Substring => "부분 문자열",
            Scope::WholeWord => "단어 단위",
        }
    }
}

fn count(text: &str, word: &str, case: Case, scope: Scope) -> usize {
    let text = case.normalize(text);
    let word = case.normalize(word);
    match scope {
        Scope::Substring => text.matches(word.as_str()).count(),
        Scope::WholeWord => text.split_whitespace().filter(|w| *w == word).count(),
    }
}

fn main() {
    println!("본문: {TEXT:?}, 찾는 말: {WORD:?}");

    println!("[비권장] bool 두 개");
    // 의도: 대소문자 무시 + 단어 단위 → (case_sensitive = false, whole_word = true)
    let intended = count_flags(TEXT, WORD, false, true);
    // 실수: 두 bool의 순서를 바꿔 넘김. 컴파일도 실행도 아무 문제 없이 됩니다.
    let swapped = count_flags(TEXT, WORD, true, false);
    println!("  의도한 호출 (false, true): {intended}");
    println!("  순서를 바꾼 호출 (true, false): {swapped}");
    // 출력:
    //   의도한 호출 (false, true): 3
    //   순서를 바꾼 호출 (true, false): 2

    println!("[권장] 열거형 두 개");
    // 호출 지점만 읽어도 무엇을 골랐는지 보입니다.
    let n = count(TEXT, WORD, Case::Insensitive, Scope::WholeWord);
    println!("  count(.., Case::Insensitive, Scope::WholeWord): {n}");
    // 출력:   count(.., Case::Insensitive, Scope::WholeWord): 3

    // 네 조합을 모두 돌려 봅니다. 결과가 모두 달라서, 순서를 바꾸면 조용히 다른 답이 나온다는 것을 알 수 있습니다.
    for case in [Case::Sensitive, Case::Insensitive] {
        for scope in [Scope::Substring, Scope::WholeWord] {
            let n = count(TEXT, WORD, case, scope);
            println!("  {} + {}: {n}", case.label(), scope.label());
        }
    }
    // 출력:
    //   대소문자 구분 + 부분 문자열: 2
    //   대소문자 구분 + 단어 단위: 1
    //   대소문자 무시 + 부분 문자열: 5
    //   대소문자 무시 + 단어 단위: 3
}
