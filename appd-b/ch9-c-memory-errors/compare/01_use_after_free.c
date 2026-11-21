// 부록 B ch9 · Q9.1 — 해제 후 사용(use-after-free)
//
// 이름을 힙에 복사해 두고, 해제한 뒤에도 같은 포인터로 읽습니다. C 컴파일러는 막지 않고, 새니타이저 없이
// 실행하면 그 자리에 무엇이 남아 있느냐에 따라 결과가 달라집니다.
// Rust에서는 해제(drop)가 소유권을 가져가므로 해제한 값을 다시 쓸 수 없습니다(examples/01_one_owner_one_free.rs).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch9-c-memory-errors
//   cc -g -fsanitize=address,undefined -o target/tmp/faq-ch9-c-memory-errors/use_after_free appd-b/ch9-c-memory-errors/compare/01_use_after_free.c
//   target/tmp/faq-ch9-c-memory-errors/use_after_free
//
// 기대하는 결과: "ERROR: AddressSanitizer: heap-use-after-free", 종료 상태 134.

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    char *name = malloc(16);
    strcpy(name, "hana");
    free(name);
    printf("%s\n", name); // 해제한 메모리를 읽음
    return 0;
}
