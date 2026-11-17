// 벡터는 요소를 소유합니다. 요소를 넣으면 소유권이 벡터로 옮겨 가고,
// 벡터가 버려질(drop) 때 남아 있던 요소도 함께 버려집니다.
//
// 실행: cargo run -p p0-ch8-collections --example 05_vec_drop

// 버려질 때 메시지를 출력하는 관찰용 타입입니다(ch4에서 쓴 것과 같습니다).
// `Drop` 트레이트는 ch14에서 배웁니다. 지금은 "버려질 때 `[drop] 이름`을 출력하는 값"으로 보면 됩니다.
struct Noisy {
    name: String,
}

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    [drop] {}", self.name);
    }
}

fn noisy(name: &str) -> Noisy {
    Noisy {
        name: String::from(name),
    }
}

fn main() {
    println!("1. push하면 값의 소유권이 벡터로 이동합니다");
    let mut v = Vec::new();
    let a = noisy("a");
    v.push(a); // 이제 "a"의 소유자는 변수 a가 아니라 벡터 v입니다.
    v.push(noisy("b"));
    v.push(noisy("c"));
    // a는 이동했으므로 더 이상 쓸 수 없습니다. `println!("{}", a.name);`은 E0382가 납니다.
    println!("    v에 {}개가 들어 있습니다", v.len()); // 출력: v에 3개가 들어 있습니다

    println!("2. pop은 마지막 요소를 꺼내 소유권을 돌려줍니다");
    // pop은 Option<Noisy>를 돌려줍니다. 비어 있으면 None입니다.
    if let Some(last) = v.pop() {
        println!("    꺼낸 요소: {}", last.name);
    } // 꺼낸 값의 새 소유자 last가 여기서 스코프를 벗어나므로 "c"가 drop됩니다.
    println!("    남은 개수: {}", v.len());
    // 출력:
    //     꺼낸 요소: c
    //     [drop] c
    //     남은 개수: 2

    println!("3. 요소 자리에 새 값을 대입하면 원래 요소는 그 자리에서 drop됩니다");
    v[0] = noisy("a2"); // "a"의 소유자(벡터의 0번 칸)가 새 값을 갖게 되므로 "a"는 버려집니다.
    println!("    0번 요소: {}", v[0].name);
    // 출력:
    //     [drop] a
    //     0번 요소: a2

    println!("4. 벡터가 drop되면 남은 요소가 앞에서부터 차례로 drop됩니다");
    {
        let mut inner = vec![noisy("x"), noisy("y")];
        inner.push(noisy("z"));
        println!("    inner 길이: {}", inner.len());
    } // inner의 스코프가 끝납니다. 벡터가 소유한 요소 x, y, z가 모두 정리됩니다.
    // 출력:
    //     inner 길이: 3
    //     [drop] x
    //     [drop] y
    //     [drop] z

    println!("5. main이 끝나면 v와 그 요소도 drop됩니다");
    // 출력:
    //     [drop] a2
    //     [drop] b
}
