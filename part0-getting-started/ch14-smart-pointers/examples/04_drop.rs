// Drop 트레이트로 값이 버려질 때 실행할 정리 코드를 정합니다.
// - drop 메서드는 Rust가 자동으로 부릅니다. 같은 스코프의 값은 만든 역순으로 버려집니다.
// - 구조체는 자기 drop이 먼저, 그다음 필드가 선언 순서대로 버려집니다. Box도 마찬가지로 안의 값을 버립니다.
// - `c.drop()`처럼 직접 부를 수는 없습니다(compile_fail/e0040_explicit_drop_call.rs).
//   일찍 버리고 싶으면 std::mem::drop 함수에 값을 넘깁니다.
//
// 실행: cargo run -p p0-ch14-smart-pointers --example 04_drop

// ch4, ch8에서 관찰용으로 쓴 Noisy와 같은 모양입니다. 이제 이 구현을 직접 읽어 봅니다.
struct CustomSmartPointer {
    data: String,
}

impl Drop for CustomSmartPointer {
    // &mut self를 받습니다. 값이 아직 살아 있는 마지막 순간에 불리므로 필드를 읽을 수 있습니다.
    fn drop(&mut self) {
        println!("    [drop] {}", self.data);
    }
}

fn custom(data: &str) -> CustomSmartPointer {
    CustomSmartPointer {
        data: String::from(data),
    }
}

// Drop을 구현한 구조체가 Drop을 구현한 필드를 가질 때의 순서를 보기 위한 타입입니다.
struct Pair {
    first: CustomSmartPointer,
    second: CustomSmartPointer,
}

impl Drop for Pair {
    fn drop(&mut self) {
        // 이 시점에는 필드가 아직 버려지지 않았으므로 읽을 수 있습니다.
        println!(
            "    [drop] Pair({}, {}) 시작",
            self.first.data, self.second.data
        );
    }
}

// std::mem::drop과 똑같이 동작하는 함수입니다. 실제 표준 라이브러리 구현도 본문이 비어 있습니다.
// 값을 넘겨받아(소유권 이동) 아무것도 하지 않고 끝나면, 매개변수가 스코프를 벗어나며 버려집니다.
fn my_drop<T>(_x: T) {}

fn main() {
    println!("1. 스코프가 끝나면 Rust가 drop을 자동으로 부릅니다 (만든 역순)");
    {
        let _c = custom("my stuff");
        let _d = custom("other stuff");
        println!("    CustomSmartPointer 두 개를 만들었습니다");
    }
    // 출력:
    //     CustomSmartPointer 두 개를 만들었습니다
    //     [drop] other stuff
    //     [drop] my stuff

    println!("2. 구조체는 자기 drop을 먼저 실행하고, 필드를 선언 순서대로 버립니다");
    {
        let _pair = Pair {
            first: custom("first"),
            second: custom("second"),
        };
    }
    // 출력:
    //     [drop] Pair(first, second) 시작
    //     [drop] first
    //     [drop] second

    println!("3. Box가 버려지면 안의 값도 drop되고, 그 뒤에 힙 메모리가 해제됩니다");
    {
        let _boxed = Box::new(custom("상자 속 값"));
        println!("    Box를 만들었습니다");
    } // Box의 Drop 구현이 안의 값을 버리고 힙 메모리를 돌려줍니다.
    // 출력:
    //     Box를 만들었습니다
    //     [drop] 상자 속 값

    println!("4. std::mem::drop으로 스코프가 끝나기 전에 버립니다");
    let c = custom("some data");
    println!("    CustomSmartPointer를 만들었습니다");
    // c.drop(); // E0040: Drop의 drop 메서드는 직접 부를 수 없습니다.
    drop(c); // std::mem::drop은 프렐류드에 있어서 use 없이 쓸 수 있습니다.
    println!("    main이 끝나기 전에 버렸습니다");
    // 출력:
    //     CustomSmartPointer를 만들었습니다
    //     [drop] some data
    //     main이 끝나기 전에 버렸습니다
    // c는 drop 함수로 이동했으므로 이 뒤에서 c를 쓰면 E0382가 납니다.
    // 그래서 main이 끝날 때 c를 한 번 더 버리는 일(이중 해제)은 일어나지 않습니다.

    println!("5. drop 함수가 하는 일은 소유권을 가져가는 것뿐입니다");
    let e = custom("my_drop으로 버린 값");
    my_drop(e); // 매개변수 _x가 my_drop의 끝에서 스코프를 벗어나며 버려집니다.
    println!("    my_drop 호출이 끝났습니다");
    // 출력:
    //     [drop] my_drop으로 버린 값
    //     my_drop 호출이 끝났습니다

    println!("6. main이 끝납니다");
    let _last = custom("마지막 값");
} // 출력: [drop] 마지막 값
