//! `.cargo/config.toml`이 실행 결과에 주는 영향을 출력합니다.
//!
//! 실행: cargo hello [이름]      (별칭. `cargo run --quiet -- [이름]`과 같습니다)

use std::env;
use std::path::Path;

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "세계".to_string());
    println!("안녕하세요, {name}");

    // [env]의 값은 실행할 때의 환경 변수로도, 컴파일할 때의 환경 변수로도 들어옵니다.
    let run = env::var("APP_MODE").unwrap_or_else(|_| "(없음)".to_string());
    let built = option_env!("APP_MODE").unwrap_or("(없음)");
    println!("APP_MODE 실행 시: {run}, 컴파일 시: {built}");

    match env::var("APP_DATA") {
        Ok(p) => {
            let p = Path::new(&p);
            let last = p.file_name().unwrap_or_default().to_string_lossy();
            println!(
                "APP_DATA: …/{last} (절대 경로: {}, sample.txt 있음: {})",
                p.is_absolute(),
                p.join("sample.txt").exists()
            );
        }
        Err(_) => println!("APP_DATA: (없음)"),
    }

    // build.rustflags의 `--cfg`가 켠 이름입니다. 어느 설정이 적용됐는지 보여 줍니다.
    println!(
        "cfg: from_config={} from_ci={} from_env={}",
        cfg!(from_config),
        cfg!(from_ci),
        cfg!(from_env)
    );
}
