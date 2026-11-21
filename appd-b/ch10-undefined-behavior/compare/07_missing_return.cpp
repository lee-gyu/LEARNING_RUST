// 부록 B ch10 · Q10.5 — 값을 돌려주지 않고 끝나는 함수
//
// grade는 점수가 80 미만이면 return 없이 함수 끝에 닿습니다. C++에서 값을 돌려줘야 하는 함수가 return 없이 끝나면
// 미정의 동작입니다. Rust에서는 모든 경로가 반환 타입의 값을 내야 하고, 빠진 경로가 있으면 컴파일 에러입니다.
// 이 함수를 그대로 옮기면 E0317(`if` may be missing an `else` clause), 마지막 줄이 문장이면 E0308입니다
// (Part 0 ch3의 "자주 하는 실수" 3).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch10-undefined-behavior
//   c++ -std=c++20 -g -fsanitize=undefined -o target/tmp/faq-ch10-undefined-behavior/missing_return appd-b/ch10-undefined-behavior/compare/07_missing_return.cpp
//   target/tmp/faq-ch10-undefined-behavior/missing_return
//
// 기대하는 결과: 컴파일할 때 "non-void function does not return a value in all control paths [-Wreturn-type]" 경고
// (기본으로 켜짐), 실행하면 "runtime error: execution reached the end of a value-returning function without returning
// a value". 이 환경에서는 다른 검사와 달리 보고 뒤 멈춰서 종료 상태가 134였습니다.

#include <cstdio>

static int grade(int score) {
    if (score >= 90) {
        return 1;
    }
    if (score >= 80) {
        return 2;
    }
} // 80 미만이면 return 없이 끝남

int main() {
    std::printf("%d\n", grade(50));
}
