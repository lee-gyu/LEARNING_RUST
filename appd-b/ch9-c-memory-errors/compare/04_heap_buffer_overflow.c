// 부록 B ch9 · Q9.4 — 경계 밖 읽기(버퍼 오버리드)
//
// 원소 4개짜리 배열을 채운 뒤, 합을 구하는 반복문에서 i <= 4로 다섯 번 읽습니다(흔한 한 칸 넘침, off-by-one).
// C의 인덱싱은 경계를 검사하지 않으므로 배열 뒤의 메모리를 그대로 읽어 합에 더합니다. 경계 밖 쓰기는
// compare/07(스택, strcpy)이 맡습니다.
// Rust의 인덱싱은 경계를 검사해 패닉으로 멈추고(ch2 예제 03), get은 None을 돌려줍니다(ch2 예제 01).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch9-c-memory-errors
//   cc -g -fsanitize=address,undefined -o target/tmp/faq-ch9-c-memory-errors/heap_buffer_overflow appd-b/ch9-c-memory-errors/compare/04_heap_buffer_overflow.c
//   target/tmp/faq-ch9-c-memory-errors/heap_buffer_overflow
//
// 기대하는 결과: "ERROR: AddressSanitizer: heap-buffer-overflow"와 "READ of size 4", 종료 상태 134.

#include <stdio.h>
#include <stdlib.h>

int main(void) {
    int *scores = malloc(4 * sizeof *scores);
    for (int i = 0; i < 4; i++) {
        scores[i] = i * 10;
    }
    int total = 0;
    for (int i = 0; i <= 4; i++) { // <가 아니라 <=
        total += scores[i];
    }
    printf("%d\n", total);
    free(scores);
    return 0;
}
