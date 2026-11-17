// 소유자(변수)가 스코프를 벗어나면 값이 버려지는(drop) 시점을 눈으로 확인합니다.
//
// 실행: cargo run -p p0-ch4-ownership --example 01_scope_and_drop

// `String`도 스코프를 벗어나면 힙 메모리를 돌려주지만, 그 과정은 아무것도 출력하지 않습니다.
// 그래서 버려질 때 메시지를 출력하는 "관찰용" 타입을 만들어 씁니다.
// 구조체(struct)는 ch5에서, `Drop` 트레이트는 ch14에서 배웁니다.
// 지금은 "버려질 때 `[drop] 이름`을 출력하는 값"으로만 이해하면 충분합니다.
struct Noisy {
    name: String,
}

impl Drop for Noisy {
    // Rust가 값을 버리기 직전에 자동으로 호출하는 함수입니다.
    // 직접 호출하는 코드는 어디에도 없다는 점에 주목하세요.
    fn drop(&mut self) {
        println!("    [drop] {}", self.name);
    }
}

// 이름을 받아 Noisy 값을 만들고, 만들었다는 사실을 출력합니다.
// `&str`(문자열 슬라이스)은 이 챕터의 뒷부분(06번 예제)에서 배웁니다.
fn noisy(name: &str) -> Noisy {
    println!("    [생성] {name}");
    Noisy {
        name: String::from(name),
    }
}

fn main() {
    println!("1. 블록이 끝나면 그 안에서 만든 값이 drop됩니다");
    {
        let a = noisy("a");
        // `a.name`은 구조체의 필드를 읽는 문법입니다(ch5).
        println!("    블록 안: {} 사용 중", a.name);
    } // ← a의 스코프가 여기서 끝나므로 Rust가 drop을 호출합니다.
    println!("    블록 밖: a라는 이름은 더 이상 존재하지 않습니다");
    // 출력:
    //     [생성] a
    //     블록 안: a 사용 중
    //     [drop] a
    //     블록 밖: a라는 이름은 더 이상 존재하지 않습니다

    println!("2. 한 스코프의 여러 값은 선언의 역순으로 drop됩니다");
    {
        let first = noisy("first");
        let second = noisy("second");
        println!("    {}, {} 사용 중", first.name, second.name);
    } // second가 먼저, first가 나중에 버려집니다.
    // 나중에 만든 값은 먼저 만든 값에 기대고 있을 수 있으므로, 역순으로 정리하는 편이 안전합니다.
    // 출력:
    //     [생성] first
    //     [생성] second
    //     first, second 사용 중
    //     [drop] second
    //     [drop] first

    println!("3. 변수에 새 값을 대입하면 이전 값은 그 자리에서 drop됩니다");
    let mut x = noisy("old");
    println!("    x = {}", x.name);
    x = noisy("new"); // "old"의 소유자가 사라졌으므로 바로 버려집니다.
    println!("    x = {}", x.name);
    // 출력:
    //     [생성] old
    //     x = old
    //     [생성] new
    //     [drop] old
    //     x = new

    println!("4. main이 끝날 때 남아 있는 값이 drop됩니다");
    // 출력:
    //     [drop] new
}
