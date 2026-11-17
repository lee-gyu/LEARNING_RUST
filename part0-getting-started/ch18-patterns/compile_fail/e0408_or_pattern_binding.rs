// 기대하는 에러: E0408 variable `side` is not bound in all patterns
//               (같은 이유로 `radius`에 대한 E0408과, 그 결과인 E0381도 함께 나옵니다)
// 원인: | 로 묶은 패턴은 어느 쪽이 맞든 갈래 안의 코드가 똑같이 실행됩니다.
//       그런데 왼쪽은 radius만, 오른쪽은 side만 묶으므로, 어느 쪽이 맞았느냐에 따라
//       갈래 안에서 쓸 수 있는 변수가 달라집니다. 그래서 | 의 모든 패턴은 같은 이름을 같은 타입으로 묶어야 합니다.
// 해결: 필드 이름을 바꿔 묶는 문법(필드: 변수)으로 양쪽 이름을 맞춥니다.
//       Shape::Circle { radius: size } | Shape::Square { side: size } => ...
//
// 확인: rustc --edition 2024 compile_fail/e0408_or_pattern_binding.rs

enum Shape {
    Circle { radius: f64 },
    Square { side: f64 },
}

fn main() {
    let shape = Shape::Circle { radius: 2.0 };

    // 원의 반지름과 정사각형의 한 변을 "크기"로 묶어 한 갈래에서 처리하려고 했습니다.
    let size = match shape {
        Shape::Circle { radius } | Shape::Square { side } => radius,
    };
    println!("크기: {size}");
}
