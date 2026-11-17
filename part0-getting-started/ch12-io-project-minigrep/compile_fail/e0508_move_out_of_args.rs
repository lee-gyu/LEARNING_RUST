// 기대하는 에러: error[E0508]: cannot move out of type `[String]`, a non-copy slice
// 핵심 메시지: move occurs because `args[_]` has type `String`, which does not implement the `Copy` trait
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0508_move_out_of_args.rs --out-dir /tmp/rust-ch12
//
// Config가 String을 소유하게 하려고 args[1]을 그대로 필드에 넣으려 한 경우입니다.
// args는 &[String]으로 빌려 온 슬라이스라서, 그 안의 String을 꺼내 옮기면(move)
// 원래 소유자인 main의 벡터에 빈자리가 생깁니다. 그래서 컴파일러가 거부합니다.
// 고치는 방법: args[1].clone()으로 복제본을 만들어 넣습니다.
// (args가 Vec<String>인 곳, 예를 들어 main에서 let q = args[1];이라고 쓰면 같은 이유로
//  E0507 "cannot move out of index of `Vec<String>`"이 납니다.)

struct Config {
    query: String,
    file_path: String,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("인자가 부족합니다");
        }

        Ok(Config {
            query: args[1],
            file_path: args[2],
        })
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if let Ok(config) = Config::build(&args) {
        println!("{} / {}", config.query, config.file_path);
    }
}
