// 기대하는 에러: E0027 pattern does not mention field `verbose`
// 원인: 구조체 패턴은 모든 필드를 적어야 합니다. 나중에 Config에 verbose 필드를 추가했는데,
//       구조 분해하는 곳을 고치지 않았습니다. 필드를 빠뜨린 것이 실수인지 의도인지 컴파일러가 알 수 없으므로
//       조용히 넘어가지 않습니다.
// 해결: 그 필드가 필요하면 패턴에 적고(Config { host, port, verbose }),
//       필요 없다면 ..로 "나머지 필드는 일부러 무시한다"고 밝힙니다(Config { host, port, .. }).
//
// 확인: rustc --edition 2024 compile_fail/e0027_missing_struct_field.rs

struct Config {
    host: String,
    port: u16,
    verbose: bool, // 새로 추가한 필드
}

fn main() {
    let config = Config {
        host: String::from("localhost"),
        port: 8080,
        verbose: false,
    };

    let Config { host, port } = config;
    println!("{host}:{port}");
}
