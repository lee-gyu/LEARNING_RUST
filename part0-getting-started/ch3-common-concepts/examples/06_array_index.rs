// 이 예제가 보여 주는 것: 배열 인덱스가 범위를 벗어나면 Rust가 실행 중에 검사해서 패닉으로 멈추는 모습
//
// 주의: 0~4가 아닌 인덱스를 입력하면 일부러 패닉하며 끝납니다(종료 코드 101).
//
// 실행(정상):  echo 2 | cargo run -p p0-ch3-common-concepts --example 06_array_index
// 실행(패닉):  echo 10 | cargo run -p p0-ch3-common-concepts --example 06_array_index
// 실행(릴리스 빌드에서도 패닉): echo 10 | cargo run --release -p p0-ch3-common-concepts --example 06_array_index

use std::io;

fn main() {
    let a = [10, 20, 30, 40, 50];

    println!("배열 인덱스를 입력하세요 (0~4).");

    // 인덱스를 표준 입력에서 받는 이유: `a[10]`처럼 컴파일러가 인덱스를 미리 알면
    // "this operation will panic at runtime"이라는 컴파일 에러로 막아 줍니다.
    // 사용자 입력처럼 실행해 봐야 아는 값은 컴파일러가 검사할 수 없으므로 실행 중에 검사합니다.
    let mut index = String::new();
    io::stdin()
        .read_line(&mut index)
        .expect("입력을 읽지 못했습니다");
    let index: usize = index.trim().parse().expect("0 이상의 정수를 입력하세요");

    // 인덱싱할 때마다 Rust는 `index < a.len()`인지 확인합니다.
    // 범위를 벗어나면 배열 뒤쪽의 엉뚱한 메모리를 읽는 대신 즉시 패닉으로 프로그램을 멈춥니다.
    // C 같은 언어에서는 이런 접근이 조용히 쓰레기 값을 읽거나 보안 취약점이 되기도 합니다.
    // 이 검사는 메모리 안전을 지키기 위한 것이어서, 정수 오버플로 검사와 달리
    // 릴리스 빌드(`--release`)에서도 꺼지지 않습니다.
    let element = a[index];

    // 패닉하면 아래 줄은 실행되지 않습니다.
    println!("a[{index}] = {element}");
    // 출력(입력 2):  a[2] = 30
    // 출력(입력 10): 패닉 — thread 'main' (스레드 ID는 실행마다 다름) panicked at ...:
    //                index out of bounds: the len is 5 but the index is 10
}
