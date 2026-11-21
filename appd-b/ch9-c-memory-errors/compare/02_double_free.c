// 부록 B ch9 · Q9.2 — 이중 해제(double free)
//
// 같은 포인터를 두 번 free합니다. 실제 코드에서는 두 곳이 "내가 해제한다"고 서로 믿을 때 생깁니다.
// Rust에서 같은 모양은 compile_fail/e0382_drop_twice.rs처럼 컴파일되지 않습니다.
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch9-c-memory-errors
//   cc -g -fsanitize=address,undefined -o target/tmp/faq-ch9-c-memory-errors/double_free appd-b/ch9-c-memory-errors/compare/02_double_free.c
//   target/tmp/faq-ch9-c-memory-errors/double_free
//
// 기대하는 결과: "ERROR: AddressSanitizer: attempting double-free", 종료 상태 134.

#include <stdlib.h>

int main(void) {
    int *count = malloc(sizeof *count);
    free(count);
    free(count); // 두 번째 해제
    return 0;
}
