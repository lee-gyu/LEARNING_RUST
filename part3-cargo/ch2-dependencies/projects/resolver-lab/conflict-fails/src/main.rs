// 호환되는 범위인데 겹치지 않아 의존성 해석이 실패하는 패키지입니다.
// 이 파일은 컴파일되기 전에 Cargo가 에러를 냅니다.
//
// 실행: cargo build   (resolver-lab/conflict-fails 디렉터리에서, 종료 코드 101)

fn main() {
    println!("내가 쓰는 tint:      {}", tint::VERSION);
    println!("banner가 쓰는 tint:  {}", banner::tint_version());
}
