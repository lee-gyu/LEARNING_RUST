// 호환되는 두 요구 사항이 버전 하나로 합쳐지는 모습을 보는 패키지입니다.
//
// 실행: cargo run -q   (resolver-lab/unified 디렉터리에서)

fn main() {
    println!("내가 쓰는 tint:      {}", tint::VERSION);
    println!("banner가 쓰는 tint:  {}", banner::tint_version());
    // 둘이 같은 크레이트이므로 내가 얻은 값과 banner 안에서 얻은 값이 같습니다.
    println!("{}", banner::line("yellow", "주의"));
    println!("tint::hex(\"yellow\") = {:?}", tint::hex("yellow"));
}
