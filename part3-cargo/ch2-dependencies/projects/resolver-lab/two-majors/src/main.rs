// 호환되지 않는 두 버전이 한 프로그램에 함께 들어가는 모습을 보는 패키지입니다.
//
// 실행: cargo run -q   (resolver-lab/two-majors 디렉터리에서)

fn main() {
    println!("내가 쓰는 tint:      {}", tint::VERSION);
    println!("banner가 쓰는 tint:  {}", banner::tint_version());
    // 같은 이름의 함수지만 서로 다른 크레이트의 함수입니다. 반환 타입도 다릅니다.
    println!("{}", banner::line("yellow", "주의"));
    println!("tint::hex(\"yellow\") = {:?}", tint::hex("yellow"));
}
