// 부록 B ch9 · Q9.3 — 끝난 함수의 지역 변수를 가리키는 포인터(댕글링 포인터)
//
// make_total은 지역 변수의 주소를 돌려줍니다. 함수가 끝나면 그 지역 변수의 자리는 다른 호출이 다시 씁니다.
// 주소를 바로 return &local; 로 돌려주면 clang이 경고하므로, 여기서는 포인터 변수를 거쳐 경고를 피한 모양으로
// 썼습니다(실제 버그도 이렇게 한 단계 떨어져 있는 경우가 많습니다).
// Rust에서 지역 변수의 참조를 돌려주면 컴파일되지 않습니다(Part 0 ch4 4.5절의 E0106, Part 0 ch10 4.5절의 E0515).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch9-c-memory-errors
//   cc -g -fsanitize=address,undefined -o target/tmp/faq-ch9-c-memory-errors/return_stack_address appd-b/ch9-c-memory-errors/compare/03_return_stack_address.c
//   ASAN_OPTIONS=detect_stack_use_after_return=1 target/tmp/faq-ch9-c-memory-errors/return_stack_address
//
// 기대하는 결과: "ERROR: AddressSanitizer: stack-use-after-return", 종료 상태 134.
// 이 검출은 macOS에서 기본으로 꺼져 있어서 ASAN_OPTIONS를 주지 않으면 보고 없이 끝납니다(Clang 문서에 따르면
// Linux에서는 기본으로 켜짐).

#include <stdio.h>

static int *make_total(void) {
    int local = 42;
    int *p = &local;
    return p; // 곧 사라질 지역 변수의 주소
}

int main(void) {
    int *total = make_total();
    printf("%d\n", *total); // 끝난 함수의 스택을 읽음
    return 0;
}
