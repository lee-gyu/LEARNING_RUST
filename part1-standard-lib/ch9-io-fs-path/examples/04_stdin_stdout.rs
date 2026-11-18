// 표준 입력을 줄 단위로 읽어 번호를 붙여 표준 출력에 쓰고, 요약과 안내는 표준 에러에 씁니다.
//
//   - 표준 입력: io::stdin().lock()은 BufRead를 구현하므로 lines()로 바로 읽을 수 있습니다.
//                파이프로 들어온 입력이 끝나면(EOF) 반복이 끝나고 프로그램도 끝납니다.
//   - 표준 출력: 줄이 많으면 io::stdout().lock()을 BufWriter로 감싸서 씁니다.
//   - 표준 에러: 결과가 아닌 메시지(요약, 안내)는 eprintln!으로 보냅니다. `>`로 결과만 파일에 담을 수 있습니다.
//   - 읽는 쪽과 쓰는 쪽을 제네릭으로 받는 함수(number_lines)는 표준 입출력에도, 메모리 버퍼에도 쓸 수 있습니다.
//
// 실행 (파이프로 입력을 넣기):
//   printf '하나 둘\n\n셋 넷 다섯\n' | cargo run -q -p p1-ch9-io-fs-path --example 04_stdin_stdout
// 결과만 파일로 보내기 (요약은 화면에 남음):
//   printf '하나 둘\n\n셋 넷 다섯\n' | cargo run -q -p p1-ch9-io-fs-path --example 04_stdin_stdout > out.txt
// 출력을 받는 쪽이 먼저 끝나는 경우:
//   yes '가 나 다' | head -n 100000 | cargo run -q -p p1-ch9-io-fs-path --example 04_stdin_stdout | head -n 2
//
// 파이프 없이 터미널에서 실행하면 입력을 기다리는 대신 내장된 예시 문장을 읽습니다.
// 입력이 비어 있으면(예: < /dev/null) 0줄로 끝납니다.

use std::io::{self, BufRead, BufWriter, IsTerminal, Write};

const SAMPLE: &str = "표준 입력이 터미널이라 예시 문장을 읽습니다\n\n파이프로 입력을 넣어 보세요\n";

#[derive(Debug, PartialEq)]
struct Stats {
    lines: usize,
    blank: usize,
    words: usize,
}

// 읽는 쪽(impl BufRead)과 쓰는 쪽(impl Write)을 모두 제네릭으로 받습니다.
// 이 함수는 입력이 표준 입력인지, 결과가 화면으로 가는지 모릅니다.
fn number_lines(input: impl BufRead, out: &mut impl Write) -> io::Result<Stats> {
    let mut stats = Stats {
        lines: 0,
        blank: 0,
        words: 0,
    };
    for line in input.lines() {
        let line = line?;
        stats.lines += 1;
        let words = line.split_whitespace().count();
        if words == 0 {
            stats.blank += 1;
        }
        stats.words += words;
        // 줄 내용은 디버그 형식으로 써서 빈 줄도 ""로 보이게 합니다.
        writeln!(out, "{:>4}: {line:?}", stats.lines)?; // println!과 달리 실패하면 Err를 돌려줍니다.
    }
    Ok(stats)
}

fn run() -> io::Result<()> {
    // 같은 함수를 메모리 버퍼로 먼저 시험해 봅니다. 결과는 표준 출력에 내지 않고 Vec<u8>에 모읍니다.
    let mut captured = Vec::new();
    let stats = number_lines("a b\n\nc\n".as_bytes(), &mut captured)?;
    assert_eq!(
        stats,
        Stats {
            lines: 3,
            blank: 1,
            words: 3
        }
    );
    assert_eq!(captured, b"   1: \"a b\"\n   2: \"\"\n   3: \"c\"\n");

    // 표준 입력이 터미널이면 사용자가 입력하기를 기다리게 되므로, 예시 문장을 대신 읽습니다.
    // is_terminal()은 IsTerminal 트레이트의 메서드로, 이 핸들이 터미널에 연결돼 있는지 알려 줍니다.
    // (터미널인지에 따라 동작을 바꾸는 설계는 Part 7에서 다룹니다.)
    let stdin = io::stdin();
    let input: Box<dyn BufRead> = if stdin.is_terminal() {
        Box::new(SAMPLE.as_bytes())
    } else {
        Box::new(stdin.lock()) // StdinLock은 BufRead를 구현합니다. 표준 입력에는 이미 버퍼가 있습니다.
    };

    // 표준 출력은 한 번 잠가 두고 BufWriter로 감쌉니다.
    let mut out = BufWriter::new(io::stdout().lock());
    let stats = number_lines(input, &mut out)?;
    out.flush()?; // 요약(표준 에러)보다 결과(표준 출력)가 먼저 나가도록 여기서 비웁니다.

    eprintln!(
        "[요약] {}줄 (빈 줄 {}개), 단어 {}개",
        stats.lines, stats.blank, stats.words
    );
    Ok(())
}

fn main() -> io::Result<()> {
    match run() {
        // `| head -n 2`처럼 출력을 받는 쪽이 먼저 끝나면 쓰기가 BrokenPipe 에러로 실패합니다.
        // 사용자가 원한 상황이므로 에러로 보고하지 않고 조용히 끝냅니다.
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
            eprintln!("[안내] 출력을 받는 쪽이 먼저 끝나서 멈춥니다");
            Ok(())
        }
        other => other,
    }
}

// 출력 (printf '하나 둘\n\n셋 넷 다섯\n' | ... 로 실행):
//    1: "하나 둘"
//    2: ""
//    3: "셋 넷 다섯"
// [요약] 3줄 (빈 줄 1개), 단어 5개
// (마지막 줄은 표준 에러입니다.)
