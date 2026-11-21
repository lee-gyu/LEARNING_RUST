// 부록 B ch13 · Q13.3 — 함수처럼 보이는 매크로가 인자를 두 번 평가함
//
// MAX는 함수처럼 보이지만 전처리기가 인자의 토큰을 본문의 자리에 그대로 바꿔 끼우므로, MAX(i++, j)는
// ((i++) > (j) ? (i++) : (j))가 됩니다.
// 조건에서 한 번, 고른 쪽에서 또 한 번 i++가 실행되어 i가 두 번 늘고, m도 처음 i(5)가 아니라 6이 됩니다.
// 조건 연산자는 첫 피연산자를 평가한 뒤에 순서점이 있으므로 미정의 동작은 아니고, 출력이 정해져 있습니다.
// -Wall -Wextra로도 경고가 없습니다.
// Rust의 macro_rules!도 본문에 인자를 두 번 쓰면 두 번 평가합니다(부록 A ch28 예제 04). 다만 Rust에서 max는
// 보통 매크로가 아니라 함수(std::cmp::max, Ord::max)이고, 함수의 인자는 한 번만 평가됩니다.
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch13-headers-build-templates
//   cc -std=c17 -Wall -Wextra -g -fsanitize=address,undefined -o target/tmp/faq-ch13-headers-build-templates/macro_double_eval appd-b/ch13-headers-build-templates/compare/01_macro_double_eval.c
//   target/tmp/faq-ch13-headers-build-templates/macro_double_eval
//
// 기대하는 결과(경고·새니타이저 보고 없음, 종료 상태 0):
//   m = 6, i = 7

#include <stdio.h>

#define MAX(a, b) ((a) > (b) ? (a) : (b))

int main(void) {
    int i = 5, j = 3;
    int m = MAX(i++, j); // i++가 두 번 실행됨
    printf("m = %d, i = %d\n", m, i);
    return 0;
}
