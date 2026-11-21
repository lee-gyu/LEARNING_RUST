//! 표준 입력의 줄·단어·글자 수를 출력합니다.
//!
//! 실행: printf '…' | cargo run -q -- [--top N]

use std::io::Read;

fn parse_top(args: &[String]) -> Option<usize> {
    let i = args.iter().position(|a| a == "--top")?;
    args.get(i + 1)?.parse().ok()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .expect("표준 입력");
    let s = linestat::count(&text);
    println!("줄 {} / 단어 {} / 글자 {}", s.lines, s.words, s.chars);

    if let Some(n) = parse_top(&args) {
        #[cfg(feature = "top")]
        for (w, c) in linestat::top_words(&text, n) {
            println!("{c:>3} {w}");
        }
        #[cfg(not(feature = "top"))]
        eprintln!("--top {n}: top 기능 없이 빌드했습니다");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_top_reads_the_next_argument() {
        let args = vec!["--top".to_string(), "3".to_string()];
        assert_eq!(parse_top(&args), Some(3));
        assert_eq!(parse_top(&[]), None);
    }
}
