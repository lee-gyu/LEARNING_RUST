//! 지침 7: 자기 참조 구조체는 만들지 않는다. 원문과 뷰를 나누거나, 범위로 가리키고 원문을 감춘다.
//!
//! 셸 명령 한 줄(`String`)과 그 안의 토큰을 한 구조체에 담고 싶습니다.
//! 토큰을 `&str`로 담으면 원문을 구조체로 옮기는 순간 E0505입니다
//! (compile_fail/e0505_self_referential_struct.rs).
//! 1번 구역: 토큰을 바이트 범위로 담되 원문을 고칠 수 있게 열어 두면, 원문을 고친 뒤 범위가
//!          엉뚱한 글자를 가리킵니다(비권장). 컴파일러는 막지 못합니다.
//! 2번 구역: 원문은 부르는 쪽이 갖고, 토큰만 빌리는 뷰를 따로 둡니다(권장 A).
//! 3번 구역: 범위로 담고 원문을 비공개로 감춰, 고칠 때는 새로 파싱하게 합니다(권장 B).
//!          이 값은 라이프타임이 없어 옮기고 돌려줘도 범위가 그대로 맞습니다.
//!
//! 실행: cargo run -p er-ch15-borrow-checker --example 08_self_referential

use std::ops::Range;

/// 공백으로 나뉜 토큰마다 바이트 범위를 구합니다.
fn token_spans(line: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut start = None;
    for (i, c) in line.char_indices() {
        match (c.is_whitespace(), start) {
            (true, Some(s)) => {
                spans.push(s..i);
                start = None;
            }
            (false, None) => start = Some(i),
            _ => {}
        }
    }
    if let Some(s) = start {
        spans.push(s..line.len());
    }
    spans
}

/// 권장 A: 원문을 빌리는 뷰. 원문은 이 값 밖에서 따로 살아 있어야 합니다.
struct CommandView<'a> {
    program: &'a str,
    args: Vec<&'a str>,
}

impl<'a> CommandView<'a> {
    fn parse(line: &'a str) -> Option<CommandView<'a>> {
        let mut words = line.split_whitespace();
        Some(CommandView {
            program: words.next()?,
            args: words.collect(),
        })
    }
}

/// 비권장: 범위로 가리키지만 원문을 누구나 고칠 수 있습니다.
struct OpenSpanned {
    line: String, // 모듈 밖에서도 고칠 수 있다고 생각하세요(pub 필드).
    spans: Vec<Range<usize>>,
}

impl OpenSpanned {
    fn tokens(&self) -> Vec<&str> {
        // 범위가 글자 경계를 가르면 get이 None을 돌려줍니다. 슬라이스 문법이었다면 패닉입니다.
        self.spans
            .iter()
            .map(|r| self.line.get(r.clone()).unwrap_or("(경계가 어긋남)"))
            .collect()
    }
}

mod command {
    use super::token_spans;
    use std::ops::Range;

    /// 권장 B: 원문과 범위를 함께 소유하고, 원문은 비공개로 감춥니다.
    /// 원문을 바꾸는 길은 새 값을 만드는 메서드뿐이라 범위가 어긋날 수 없습니다.
    pub struct Command {
        line: String,
        spans: Vec<Range<usize>>,
    }

    impl Command {
        pub fn parse(line: String) -> Command {
            let spans = token_spans(&line);
            Command { line, spans }
        }

        pub fn program(&self) -> Option<&str> {
            self.spans.first().map(|r| &self.line[r.clone()])
        }

        pub fn args(&self) -> Vec<&str> {
            self.spans
                .iter()
                .skip(1)
                .map(|r| &self.line[r.clone()])
                .collect()
        }

        /// 앞에 단어를 붙인 새 명령. 원문이 바뀌므로 범위도 새로 구합니다.
        pub fn with_prefix(&self, word: &str) -> Command {
            Command::parse(format!("{word} {}", self.line))
        }
    }
}

use command::Command;

/// 라이프타임이 없는 값이라 함수에서 만들어 돌려줄 수 있습니다.
/// 같은 일을 CommandView로 하면 원문이 이 함수의 지역 값이라 E0515입니다.
fn load(raw: &str) -> Command {
    Command::parse(raw.trim().to_string())
}

fn main() {
    let raw = "cp 사진.jpg 백업/";

    println!("[비권장] 범위로 가리키되 원문을 고칠 수 있게 열어 둠");
    let mut open = OpenSpanned {
        line: raw.to_string(),
        spans: token_spans(raw),
    };
    println!("  고치기 전: {:?}", open.tokens());
    // 출력:   고치기 전: ["cp", "사진.jpg", "백업/"]
    open.line.insert_str(0, "sudo "); // 범위는 그대로
    println!("  원문 앞에 \"sudo \"를 붙인 뒤: {:?}", open.tokens());
    // 출력:   원문 앞에 "sudo "를 붙인 뒤: ["su", "(경계가 어긋남)", "(경계가 어긋남)"]

    println!("[권장 A] 원문과 뷰를 나눔: 원문은 부르는 쪽이 소유");
    let line = String::from(raw);
    if let Some(view) = CommandView::parse(&line) {
        println!("  program = {}, args = {:?}", view.program, view.args);
    }
    // 출력:   program = cp, args = ["사진.jpg", "백업/"]

    println!("[권장 B] 범위로 가리키고 원문은 감춤");
    let mut history: Vec<Command> = Vec::new();
    history.push(Command::parse(raw.to_string())); // 구조체를 옮겨도 범위는 원문 기준이라 그대로 맞음
    history.push(load("  ls -l 백업/  "));
    let sudo = history[0].with_prefix("sudo");
    history.push(sudo);
    for cmd in &history {
        println!("  program = {:?}, args = {:?}", cmd.program(), cmd.args());
    }
    // 출력:   program = Some("cp"), args = ["사진.jpg", "백업/"]
    // 출력:   program = Some("ls"), args = ["-l", "백업/"]
    // 출력:   program = Some("sudo"), args = ["cp", "사진.jpg", "백업/"]
}
