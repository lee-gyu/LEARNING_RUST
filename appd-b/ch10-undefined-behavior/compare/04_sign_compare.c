// 부록 B ch10 · Q10.4 — 정수 승격과 부호가 다른 비교(결과가 정해진 경우)
//
// 미정의 동작은 아니지만 예상과 다른 결과가 정해져 있는 경우입니다. int와 unsigned를 비교하면 int 쪽이 unsigned로
// 바뀌어 -1은 아주 큰 수가 되고, unsigned char끼리 더하면 먼저 int로 승격되어 255를 넘는 300이 나옵니다.
// Rust에서 앞의 비교는 E0308로 컴파일되지 않고(compile_fail/e0308_compare_signed_unsigned.rs), u8끼리의 덧셈은
// u8 범위에서 계산됩니다(examples/02_conversions_and_bits.rs).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch10-undefined-behavior
//   cc -Wall -Wextra -o target/tmp/faq-ch10-undefined-behavior/sign_compare appd-b/ch10-undefined-behavior/compare/04_sign_compare.c
//   target/tmp/faq-ch10-undefined-behavior/sign_compare
//
// 기대하는 결과: 컴파일할 때 "comparison of integers of different signs: 'int' and 'unsigned int' [-Wsign-compare]"
// 경고(-Wextra가 있어야 나옴). 실행 출력은 정해져 있습니다:
//   -1 < 1u: 0
//   200 + 100 (unsigned char): 300

#include <stdio.h>

int main(void) {
    int balance = -1;
    unsigned limit = 1;
    printf("-1 < 1u: %d\n", balance < limit);

    unsigned char a = 200, b = 100;
    int sum = a + b; // a와 b가 int로 승격된 뒤 더함
    printf("200 + 100 (unsigned char): %d\n", sum);
    return 0;
}
