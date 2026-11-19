// 의존성 smol_str의 SmolStr을 써 보는 작은 프로그램입니다. 이 프로젝트에서 볼 것은 실행 결과보다
// Cargo가 smol_str의 몇 번 버전을 골랐는지입니다(Cargo.toml, Cargo.lock, README 2.6절).
//
// SmolStr은 짧은 문자열(23바이트까지)을 힙 할당 없이 값 안에 바로 담는 문자열 타입입니다.
//
// 실행(projects/msrv_resolver 디렉터리에서): cargo run -q

use smol_str::SmolStr;

fn main() {
    for text in ["서울", "이 문장은 스물세 바이트보다 훨씬 깁니다"] {
        let s = SmolStr::new(text);
        println!(
            "{s:?}: {}바이트, 힙에 할당했나: {}",
            s.len(),
            s.is_heap_allocated()
        );
    }
}
