// 부록 B ch9 · Q9.8 — strcpy로 작은 버퍼 넘치기
//
// greet는 8바이트 버퍼에 길이를 확인하지 않고 이름을 복사합니다. strcpy는 끝의 0 바이트가 나올 때까지
// 복사하므로 14바이트(0 바이트 포함)가 8바이트 칸을 넘어 스택의 다른 값을 덮어씁니다.
// 상수 문자열을 바로 strcpy하면 clang이 컴파일할 때 경고(-Wfortify-source)하므로, 함수 인자로 넘겨 경고가
// 나지 않는 모양으로 썼습니다. Rust로 옮긴 판은 examples/02_buffers_and_strings.rs의 1번입니다.
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch9-c-memory-errors
//   cc -g -fsanitize=address,undefined -o target/tmp/faq-ch9-c-memory-errors/strcpy_overflow appd-b/ch9-c-memory-errors/compare/07_strcpy_overflow.c
//   target/tmp/faq-ch9-c-memory-errors/strcpy_overflow
//
// 기대하는 결과: "ERROR: AddressSanitizer: stack-buffer-overflow", 종료 상태 134.

#include <stdio.h>
#include <string.h>

static void greet(const char *user) {
    char name[8];
    strcpy(name, user); // 길이를 확인하지 않음
    printf("hello, %s\n", name);
}

int main(void) {
    greet("hana-kim-long");
    return 0;
}
