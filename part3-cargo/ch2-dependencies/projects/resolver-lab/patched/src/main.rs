// `[patch]`로 의존성을 고친 사본으로 바꿔 끼운 패키지입니다. unified/와 소스가 같습니다.
//
// 실행: cargo run -q   (resolver-lab/patched 디렉터리에서)

fn main() {
    println!("내가 쓰는 tint:      {}", tint::VERSION);
    println!("banner가 쓰는 tint:  {}", banner::tint_version());
    println!("{}", banner::line("yellow", "주의"));
    println!("tint::hex(\"yellow\") = {:?}", tint::hex("yellow"));
}
