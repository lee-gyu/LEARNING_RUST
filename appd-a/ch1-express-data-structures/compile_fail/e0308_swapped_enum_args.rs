// 기대하는 에러: E0308 — arguments to this function are incorrect
//   (expected `Case`, found `Scope` / expected `Scope`, found `Case`, help: swap these arguments)
//
// 확인: rustc --edition 2024 compile_fail/e0308_swapped_enum_args.rs
//
// examples/03_bool_args_to_enum.rs의 count를 부르면서 두 선택지의 순서를 바꿨습니다.
// bool 두 개였다면 조용히 다른 답을 냈을 실수가, 열거형 두 개에서는 컴파일 에러가 됩니다.

#[derive(Clone, Copy)]
enum Case {
    Sensitive,
    Insensitive,
}

#[derive(Clone, Copy)]
enum Scope {
    Substring,
    WholeWord,
}

fn count(text: &str, word: &str, case: Case, scope: Scope) -> usize {
    let (text, word) = match case {
        Case::Sensitive => (text.to_string(), word.to_string()),
        Case::Insensitive => (text.to_lowercase(), word.to_lowercase()),
    };
    match scope {
        Scope::Substring => text.matches(word.as_str()).count(),
        Scope::WholeWord => text.split_whitespace().filter(|w| *w == word).count(),
    }
}

fn main() {
    let n = count("Rust rust", "rust", Scope::WholeWord, Case::Insensitive); // E0308
    println!("{n}");
}
