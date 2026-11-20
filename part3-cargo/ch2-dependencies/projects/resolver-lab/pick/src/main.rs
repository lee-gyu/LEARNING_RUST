// 요구 사항 하나로 Cargo가 어느 버전을 고르는지 보는 패키지입니다.
//
// 실행: cargo run -q   (resolver-lab/pick 디렉터리에서)

fn main() {
    println!("Cargo가 고른 tint: {}", tint::VERSION);
    println!("red = {:?}", tint::hex("red"));
}
