// 부록 B ch14 · Q14.9 — C에서도 꼬리 호출 최적화는 최적화 수준에 달려 있음
//
// count_tail은 examples/01_recursion_depth.rs의 count_tail과 같은 꼬리 재귀입니다. C 표준도 꼬리 호출을 반복문으로
// 바꾼다고 약속하지 않습니다. clang은 -O2에서 이것을 반복문으로 바꿔 1000만 번을 끝내지만, -O0에서는 호출마다
// 스택 프레임이 쌓여 메인 스레드의 스택이 넘치고 SIGSEGV(종료 상태 139)로 끝납니다. ASan을 켜면 스택 오버플로를
// "stack-overflow"로 보고합니다(종료 상태 134). 같은 일이 Rust의 디버그·릴리스 빌드에서도 일어나는 것은
// 두 컴파일러가 같은 LLVM 최적화를 쓰기 때문입니다.
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch14-performance
//   cc -std=c17 -Wall -Wextra -O2 -o target/tmp/faq-ch14-performance/deep_o2 appd-b/ch14-performance/compare/01_deep_recursion.c
//   target/tmp/faq-ch14-performance/deep_o2            → 10000000, 종료 상태 0
//   cc -std=c17 -Wall -Wextra -O0 -o target/tmp/faq-ch14-performance/deep_o0 appd-b/ch14-performance/compare/01_deep_recursion.c
//   target/tmp/faq-ch14-performance/deep_o0            → 출력 없이 종료 상태 139(SIGSEGV)
//   cc -std=c17 -O0 -g -fsanitize=address,undefined -o target/tmp/faq-ch14-performance/deep_asan appd-b/ch14-performance/compare/01_deep_recursion.c
//   target/tmp/faq-ch14-performance/deep_asan          → "ERROR: AddressSanitizer: stack-overflow", 종료 상태 134

#include <stdio.h>

static unsigned long count_tail(unsigned long n, unsigned long acc) {
    if (n == 0) {
        return acc;
    }
    return count_tail(n - 1, acc + 1);
}

int main(void) {
    printf("%lu\n", count_tail(10000000UL, 0));
    return 0;
}
