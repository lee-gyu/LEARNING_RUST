// cargo test의 실행 방식을 바꾸는 옵션을 실험하는 예제입니다.
// 출력 캡처와 --show-output, 이름으로 거르기, #[ignore]와 --ignored, 병렬 실행과 --test-threads를 다룹니다.
//
// main 실행:   cargo run -p p0-ch11-testing --example 04_running_options
// 테스트 실행: cargo test -p p0-ch11-testing --example 04_running_options
//
// `--` 앞은 cargo에게, 뒤는 테스트 바이너리에게 주는 옵션입니다. 예:
//   cargo test -p p0-ch11-testing --example 04_running_options -- --show-output
//   cargo test -p p0-ch11-testing --example 04_running_options one_hundred
//   cargo test -p p0-ch11-testing --example 04_running_options -- --ignored
//   cargo test -p p0-ch11-testing --example 04_running_options shared_file -- --ignored --test-threads=1

fn add_two(a: i32) -> i32 {
    // 테스트 중에 println!으로 찍은 내용은 기본적으로 테스트 러너가 가로채 둡니다(캡처).
    // 통과한 테스트의 출력은 버리고, 실패한 테스트의 출력만 실패 메시지와 함께 보여 줍니다.
    println!("add_two 호출: a = {a}");
    a + 2
}

fn main() {
    let result = add_two(100); // 출력: add_two 호출: a = 100
    println!("결과: {result}"); // 출력: 결과: 102
}

#[cfg(test)]
mod tests {
    use super::*;

    // 테스트 이름은 모듈 경로를 포함합니다: tests::add_two_and_two
    // cargo test 뒤에 쓴 문자열이 이름에 들어 있는 테스트만 실행됩니다.
    // "add"를 주면 아래 두 테스트가, "one_hundred"를 주면 세 번째 테스트만 실행됩니다.
    #[test]
    fn add_two_and_two() {
        assert_eq!(add_two(2), 4);
    }

    #[test]
    fn add_three_and_two() {
        assert_eq!(add_two(3), 5);
    }

    #[test]
    fn one_hundred() {
        assert_eq!(add_two(100), 102);
    }

    // 오래 걸리는 테스트를 흉내 냅니다. 1초 동안 멈춰 있다가 끝납니다.
    // #[ignore]가 붙은 테스트는 평소에 건너뛰고, -- --ignored나 -- --include-ignored를 줄 때만 실행합니다.
    #[test]
    #[ignore = "오래 걸림"]
    fn expensive_test() {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // 테스트는 기본적으로 여러 스레드에서 동시에 실행됩니다.
    // 아래 두 테스트는 같은 파일 하나를 함께 쓰므로, 동시에 실행되면 서로의 내용을 덮어씁니다.
    // 코드에는 잘못이 없는데 실행 순서에 따라 실패하는, 공유 상태 때문에 생기는 문제를 보여 주려고 일부러 이렇게 만들었습니다.
    // 평소 실행에서 실패하지 않도록 #[ignore]를 붙였습니다.
    mod shared_file {
        use std::env;
        use std::fs;
        use std::thread;
        use std::time::Duration;

        // 임시 디렉터리(env::temp_dir())의 같은 파일에 내용을 쓰고, 잠시 기다렸다가 다시 읽습니다.
        // 기다리는 사이에 다른 테스트가 같은 파일에 쓰면 처음 쓴 것과 다른 내용을 읽게 됩니다.
        // 파일은 지우지 않고 임시 디렉터리에 남겨 둡니다.
        fn write_then_read(content: &str) -> String {
            let path = env::temp_dir().join("p0-ch11-testing-shared.txt");
            fs::write(&path, content).unwrap();
            println!("파일에 쓴 값: {content}");

            thread::sleep(Duration::from_millis(200));

            let read_back = fs::read_to_string(&path).unwrap();
            println!("파일에서 읽은 값: {read_back}");
            read_back
        }

        #[test]
        #[ignore = "병렬 실행 데모"]
        fn writes_a() {
            assert_eq!(write_then_read("A"), "A");
        }

        #[test]
        #[ignore = "병렬 실행 데모"]
        fn writes_b() {
            assert_eq!(write_then_read("B"), "B");
        }
    }
}
