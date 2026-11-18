// 파이프(pipe)로 자식 프로세스와 데이터를 주고받습니다.
//
//   1) Stdio::piped()로 자식의 표준 입력에 쓰고 표준 출력을 읽습니다.
//      표준 입력을 닫아야(drop) 자식이 입력의 끝(EOF)을 알 수 있습니다.
//   2) 큰 데이터는 쓰는 일을 다른 스레드에 맡겨 교착 상태를 피합니다.
//   3) Stdio::null()은 입력을 비우거나 출력을 버립니다. output()은 표준 입력을 빈 입력으로 줍니다.
//   4) 한 자식의 표준 출력을 다른 자식의 표준 입력에 이어 셸의 `a | b`를 만듭니다.
//   5) (macOS/Linux) 같은 방법으로 sort와 uniq를 잇습니다.
//
// 실행: cargo run -p p1-ch10-env-process-time --example 04_pipes
//
// 1번 구역의 "[자식] ... 받음" 줄은 자식이 부모와 공유하는 표준 에러에 직접 쓴 것입니다.
// 5번 구역만 유닉스 명령(sort, uniq)을 씁니다.

use std::env;
use std::io::{self, BufRead, Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::thread;

// ---- 자식 모드 ----

fn child_main(mode: &str) -> io::Result<()> {
    match mode {
        // 표준 입력을 끝까지 읽어 대문자로 바꿔 표준 출력에 쓰고, 받은 양을 표준 에러에 알립니다.
        "upper" => {
            let mut text = String::new();
            io::stdin().read_to_string(&mut text)?;
            print!("{}", text.to_uppercase());
            eprintln!(
                "[자식] {}줄, {}바이트 받음",
                text.lines().count(),
                text.len()
            );
        }
        // 표준 입력을 그대로 표준 출력에 복사합니다.
        "echo" => {
            io::copy(&mut io::stdin().lock(), &mut io::stdout().lock())?;
        }
        "numbers" => {
            for n in 1..=5 {
                println!("{n}");
            }
        }
        // 한 줄에 숫자 하나씩 읽어 합계를 냅니다.
        "sum" => {
            let mut total = 0;
            for line in io::stdin().lock().lines() {
                total += line?.trim().parse::<i64>().unwrap_or(0);
            }
            println!("합계 {total}");
        }
        other => eprintln!("알 수 없는 모드: {other}"),
    }
    Ok(())
}

// ---- 부모 ----

fn child(mode: &str) -> Command {
    let exe = env::current_exe().expect("실행 파일 경로를 알 수 있어야 합니다");
    let mut cmd = Command::new(exe);
    cmd.args(["child", mode]);
    cmd
}

fn run_parent() -> io::Result<()> {
    println!("1. 자식의 표준 입력에 쓰고, 표준 출력을 읽습니다");
    let mut upper = child("upper")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    // child.stdin은 Option<ChildStdin>입니다. take()로 꺼내야 child를 계속 쓸 수 있습니다.
    let mut stdin = upper.stdin.take().expect("stdin을 piped로 설정했습니다");
    stdin.write_all("hello\nrust\n안녕\n".as_bytes())?;
    // 쓰는 쪽을 닫아야 자식의 read_to_string이 끝납니다. 닫지 않으면 자식은 입력을 계속 기다립니다.
    drop(stdin);
    // wait_with_output은 남은 표준 출력을 끝까지 읽고 자식이 끝나기를 기다립니다.
    let output = upper.wait_with_output()?;
    println!(
        "    받은 출력: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    // 출력: [자식] 3줄, 18바이트 받음              ← 자식이 표준 에러에 직접 쓴 줄
    // 출력:     받은 출력: "HELLO\nRUST\n안녕\n"

    println!("2. 큰 데이터는 다른 스레드에서 씁니다");
    let mut echo = child("echo")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = echo.stdin.take().expect("stdin을 piped로 설정했습니다");
    let data: String = (0..100_000).map(|i| format!("줄 {i}\n")).collect();
    let sent = data.len();
    // 같은 스레드에서 모두 쓰고 나서 읽으면, 자식의 출력 파이프가 가득 찬 순간 자식은 쓰기를 멈추고,
    // 자식이 더 읽지 않으니 부모의 쓰기도 멈춥니다(교착 상태). 쓰기를 다른 스레드에 맡기면
    // 부모는 동시에 출력을 읽을 수 있습니다. stdin은 스레드가 끝날 때 drop되어 닫힙니다.
    let writer = thread::spawn(move || stdin.write_all(data.as_bytes()));
    let output = echo.wait_with_output()?;
    writer
        .join()
        .expect("쓰기 스레드가 패닉하지 않아야 합니다")?;
    println!("    보낸 {sent}바이트, 받은 {}바이트", output.stdout.len()); // 출력:     보낸 988890바이트, 받은 988890바이트

    println!("3. Stdio::null과 output()의 표준 입력");
    let quiet = child("upper")
        .stdin(Stdio::null()) // 읽으면 바로 입력의 끝
        .stderr(Stdio::null()) // 쓴 것은 버려짐
        .output()?;
    println!(
        "    stdin·stderr를 null로: 출력 {:?}",
        String::from_utf8_lossy(&quiet.stdout)
    ); // 출력:     stdin·stderr를 null로: 출력 ""
    // output()은 표준 출력과 표준 에러를 모아 오고, 표준 입력은 빈 입력으로 줍니다.
    let captured = child("upper").output()?;
    println!(
        "    output()의 표준 에러: {:?}",
        String::from_utf8_lossy(&captured.stderr)
    ); // 출력:     output()의 표준 에러: "[자식] 0줄, 0바이트 받음\n"

    println!("4. 두 자식을 파이프로 잇습니다 (numbers | sum)");
    let mut producer = child("numbers").stdout(Stdio::piped()).spawn()?;
    let pipe = producer
        .stdout
        .take()
        .expect("stdout을 piped로 설정했습니다");
    // ChildStdout을 Stdio로 바꿔 다음 자식의 표준 입력으로 넘깁니다. 부모는 이 데이터를 만지지 않습니다.
    let consumer = child("sum")
        .stdin(Stdio::from(pipe))
        .stdout(Stdio::piped())
        .spawn()?;
    let result = consumer.wait_with_output()?;
    let producer_status = producer.wait()?; // 앞쪽 자식도 기다려야 합니다
    println!("    {}", String::from_utf8_lossy(&result.stdout).trim_end()); // 출력:     합계 15
    println!("    앞쪽: {producer_status}, 뒤쪽: {}", result.status); // 출력:     앞쪽: exit status: 0, 뒤쪽: exit status: 0

    #[cfg(unix)]
    {
        println!("5. (macOS/Linux) sort | uniq");
        let mut sort = Command::new("sort")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut sort_in = sort.stdin.take().expect("stdin을 piped로 설정했습니다");
        // 작은 데이터라 파이프 버퍼에 다 들어가므로 같은 스레드에서 써도 됩니다.
        sort_in.write_all(b"banana\napple\ncherry\napple\n")?;
        drop(sort_in);
        let sorted = sort.stdout.take().expect("stdout을 piped로 설정했습니다");
        let uniq = Command::new("uniq")
            .stdin(Stdio::from(sorted))
            .stdout(Stdio::piped())
            .spawn()?;
        let output = uniq.wait_with_output()?;
        sort.wait()?;
        println!("    {:?}", String::from_utf8_lossy(&output.stdout)); // 출력:     "apple\nbanana\ncherry\n"
    }

    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let result = if args.len() == 3 && args[1] == "child" {
        child_main(&args[2])
    } else {
        run_parent()
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("에러: {e}");
            ExitCode::FAILURE
        }
    }
}
