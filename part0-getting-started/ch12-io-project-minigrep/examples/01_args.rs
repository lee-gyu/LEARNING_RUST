// minigrep 1단계: 명령줄 인자를 벡터로 모아서 살펴봅니다.
//
// 실행 (저장소 루트에서):
//   cargo run -q -p p0-ch12-io-project-minigrep --example 01_args -- needle haystack
//   cargo run -q -p p0-ch12-io-project-minigrep --example 01_args
//
// `--` 뒤에 쓴 것은 cargo가 해석하지 않고 그대로 이 프로그램에 넘깁니다.

use std::env;

fn main() {
    // env::args()는 인자를 하나씩 꺼내 주는 반복자(iterator)를 돌려주고,
    // collect()가 그 값들을 모아 컬렉션 하나로 만듭니다. collect()는 벡터 말고도 여러
    // 컬렉션을 만들 수 있어서 어떤 것을 만들지 스스로 정하지 못하므로, 타입을 적어 줍니다.
    // 반복자와 collect()는 ch13에서 자세히 다룹니다.
    let args: Vec<String> = env::args().collect();

    // 인자 없이 실행해도 길이는 0이 아니라 1입니다. 0번 자리는 늘 실행 파일 경로입니다.
    println!("인자 개수: {}", args.len()); // 출력: 인자 개수: 3 (needle haystack을 넘긴 경우)

    // 0번은 운영체제가 넣어 주는 실행 파일 경로라서 실행 위치에 따라 달라집니다.
    // 저장소 루트에서 실행하면 "target/debug/examples/01_args"처럼 현재 디렉터리 기준의
    // 상대 경로가, 다른 디렉터리에서 실행하면 절대 경로가 나옵니다.
    println!("args[0] = {:?}", args[0]); // 출력: args[0] = "target/debug/examples/01_args" (실행 위치에 따라 다름)

    // 사용자가 넘긴 진짜 인자는 1번부터입니다. &args[1..]은 1번부터 끝까지의 슬라이스이며,
    // 인자가 하나도 없으면(길이 1) 빈 슬라이스가 되므로 패닉이 나지 않습니다.
    println!("사용자가 넘긴 인자: {:?}", &args[1..]); // 출력: 사용자가 넘긴 인자: ["needle", "haystack"]
}
