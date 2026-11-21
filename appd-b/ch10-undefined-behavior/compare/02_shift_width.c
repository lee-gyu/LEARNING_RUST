// 부록 B ch10 · Q10.5 — 폭 이상으로 시프트하기
//
// 32비트 unsigned를 32만큼 시프트합니다. C에서는 시프트 양이 왼쪽 피연산자의 폭 이상이면 미정의 동작입니다.
// Rust는 checked_shl / wrapping_shl로 결과를 고르고, << 연산자는 디버그에서 패닉, 릴리스에서 시프트 양을 폭으로
// 나눈 나머지만큼 밉니다(examples/01_defined_arithmetic.rs의 1번).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch10-undefined-behavior
//   cc -g -fsanitize=undefined -o target/tmp/faq-ch10-undefined-behavior/shift_width appd-b/ch10-undefined-behavior/compare/02_shift_width.c
//   target/tmp/faq-ch10-undefined-behavior/shift_width
//
// 기대하는 결과: "runtime error: shift exponent 32 is too large for 32-bit type 'unsigned int'", 종료 상태 0.

#include <stdio.h>

static unsigned mask(unsigned bits) {
    return 1u << bits; // bits가 32면 미정의 동작
}

int main(void) {
    printf("%u\n", mask(32));
    return 0;
}
