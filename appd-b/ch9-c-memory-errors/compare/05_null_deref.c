// 부록 B ch9 · Q9.6 — 널 포인터 역참조
//
// find는 못 찾으면 NULL을 돌려주는데, 부르는 쪽이 확인하지 않고 필드를 읽습니다.
// Rust의 참조는 널이 될 수 없고(compile_fail/e0308_null_into_reference.rs), 없을 수 있는 값은 Option으로
// 돌려줘 확인을 강제합니다(ch5 Q5.1).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch9-c-memory-errors
//   cc -g -fsanitize=address,undefined -o target/tmp/faq-ch9-c-memory-errors/null_deref appd-b/ch9-c-memory-errors/compare/05_null_deref.c
//   target/tmp/faq-ch9-c-memory-errors/null_deref
//
// 기대하는 결과: UndefinedBehaviorSanitizer가 "runtime error: member access within null pointer"를 먼저 알리고,
// 이어서 "ERROR: AddressSanitizer: SEGV on unknown address 0x000000000000", 종료 상태 134.

#include <stdio.h>

struct node {
    int value;
};

static struct node *find(int id) {
    (void)id;
    return NULL; // 못 찾음
}

int main(void) {
    struct node *n = find(3);
    printf("%d\n", n->value); // NULL인지 확인하지 않음
    return 0;
}
