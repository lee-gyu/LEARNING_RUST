// 부록 B ch10 · Q10.1 — 초기화하지 않은 지역 변수 읽기
//
// 인자가 5개를 넘지 않으면 total에 아무 값도 넣지 않은 채 읽습니다. 그 자리에 남아 있던 비트가 무엇이냐에 따라
// 결과가 달라지고, 컴파일러의 가정에 따라 더 이상하게 동작할 수도 있습니다. AddressSanitizer와 UBSan은 이것을
// 검사하지 않고, 검사하는 MemorySanitizer는 이 Apple clang에서 쓸 수 없습니다(ch9 Q9.5).
// Rust에서는 초기화하지 않았을 수 있는 변수를 읽으면 E0381로 컴파일되지 않습니다(ch2 4.1절).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch10-undefined-behavior
//   cc -Wall -o target/tmp/faq-ch10-undefined-behavior/uninit_read appd-b/ch10-undefined-behavior/compare/05_uninit_read.c
//
// 기대하는 결과: 컴파일할 때 "variable 'total' is used uninitialized whenever 'if' condition is false
// [-Wsometimes-uninitialized]" 경고(-Wall이 있어야 나옴). 실행 결과는 정해지지 않으므로 README에 싣지 않습니다.

#include <stdio.h>

int main(int argc, char **argv) {
    (void)argv;
    int total;
    if (argc > 5) {
        total = 1;
    }
    printf("%d\n", total); // argc <= 5이면 초기화하지 않은 값을 읽음
    return 0;
}
