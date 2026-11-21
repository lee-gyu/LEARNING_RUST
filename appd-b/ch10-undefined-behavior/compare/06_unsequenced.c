// 부록 B ch10 · Q10.6 — 순서가 정해지지 않은 두 수정
//
// i = i++ + 1은 한 식 안에서 i를 두 번 고치는데, 두 수정 사이의 순서를 C 표준이 정하지 않으므로 미정의 동작입니다.
// Rust에는 ++가 없고(ch7 Q7.2), 피연산자는 왼쪽에서 오른쪽으로 평가된다고 정해져 있습니다
// (examples/03_evaluation_order.rs).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch10-undefined-behavior
//   cc -o target/tmp/faq-ch10-undefined-behavior/unsequenced appd-b/ch10-undefined-behavior/compare/06_unsequenced.c
//
// 기대하는 결과: 컴파일할 때 "multiple unsequenced modifications to 'i' [-Wunsequenced]" 경고(기본으로 켜짐).
// 실행 결과는 정해지지 않으므로 README에 싣지 않습니다.

#include <stdio.h>

int main(void) {
    int i = 1;
    i = i++ + 1;
    printf("%d\n", i);
    return 0;
}
