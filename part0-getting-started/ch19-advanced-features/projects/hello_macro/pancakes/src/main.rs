// derive 매크로를 쓰는 쪽입니다. HelloMacro를 직접 구현하지 않습니다.
//
// 실행(projects/hello_macro 디렉터리에서): cargo run

// 이름이 같은 두 가지를 가져옵니다. 트레이트와 매크로는 서로 다른 이름 공간에 있어 충돌하지 않습니다.
use hello_macro::HelloMacro; // 트레이트: Pancakes::hello_macro()를 부르려면 스코프에 있어야 합니다(ch10).
use hello_macro_derive::HelloMacro; // derive 매크로: #[derive(HelloMacro)]에 쓰입니다.

// derive가 만든 `impl HelloMacro for Pancakes { ... }`가 이 자리 바로 뒤에 덧붙여집니다.
// 만들어진 코드는 이 파일 안에서 해석되므로, 그 코드가 쓰는 HelloMacro라는 이름도
// 위의 use로 가져온 트레이트를 가리킵니다.
#[derive(HelloMacro)]
struct Pancakes;

// 구현이 타입 이름만 쓰므로 어떤 타입에든 붙일 수 있습니다.
#[derive(HelloMacro)]
struct Waffles;

fn main() {
    Pancakes::hello_macro(); // 출력: 안녕하세요, 매크로! 제 이름은 Pancakes입니다!
    Waffles::hello_macro(); // 출력: 안녕하세요, 매크로! 제 이름은 Waffles입니다!
}
