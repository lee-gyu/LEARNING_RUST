// 부록 B ch10 · Q10.5 — 0으로 나누기와 INT_MIN / -1
//
// 두 나눗셈 모두 C에서는 미정의 동작입니다. INT_MIN / -1의 수학적 결과 2147483648은 int에 들어가지 않습니다.
// Rust에서는 둘 다 빌드와 상관없이 패닉하고, checked_div 등으로 결과를 고를 수 있습니다
// (examples/01_defined_arithmetic.rs의 2번).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch10-undefined-behavior
//   cc -g -fsanitize=undefined -o target/tmp/faq-ch10-undefined-behavior/division appd-b/ch10-undefined-behavior/compare/03_division.c
//   target/tmp/faq-ch10-undefined-behavior/division
//
// 기대하는 결과: "runtime error: division of -2147483648 by -1 cannot be represented in type 'int'"와
// "runtime error: division by zero", 종료 상태 0. (이 환경에서는 두 나눗셈을 한 함수의 같은 위치에 두면 첫 번째만
// 보고되어, 두 나눗셈을 다른 함수에 두었습니다.)

#include <limits.h>
#include <stdio.h>

static int flip(int value, int sign) {
    return value / sign;
}

static int average(int total, int count) {
    return total / count;
}

int main(void) {
    printf("%d\n", flip(INT_MIN, -1));
    printf("%d\n", average(10, 0));
    return 0;
}
