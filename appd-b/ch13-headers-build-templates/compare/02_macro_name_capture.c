// 부록 B ch13 · Q13.3 — 매크로 안의 지역 변수가 부르는 쪽의 같은 이름을 가로챔
//
// PRINT_TWICE는 반복 횟수를 세려고 자기 지역 변수 count를 만듭니다. 부르는 쪽이 마침 count라는 변수를
// 넘기면, 펼친 코드 안에서 count는 매크로의 count를 가리키게 되어 7을 두 번이 아니라 1과 0을 찍습니다.
// 전처리기는 이름이 어디서 왔는지 따지지 않고 토큰을 바꿔 끼우기 때문입니다(매크로 위생이 없음).
// 미정의 동작은 없고 출력이 정해져 있습니다. 컴파일러의 단서는 -Wall이 내는 "unused variable 'count'"
// (바깥 count를 아무도 읽지 않음) 하나이고, -Wshadow를 더 켜면 "declaration shadows a local variable"도 납니다.
// Rust의 macro_rules!에서는 매크로 안의 지역 변수가 부르는 쪽의 같은 이름과 섞이지 않습니다(부록 A ch28 2.4절).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch13-headers-build-templates
//   cc -std=c17 -Wall -Wextra -g -fsanitize=address,undefined -o target/tmp/faq-ch13-headers-build-templates/macro_name_capture appd-b/ch13-headers-build-templates/compare/02_macro_name_capture.c
//   target/tmp/faq-ch13-headers-build-templates/macro_name_capture
//
// 기대하는 결과(경고 "unused variable 'count' [-Wunused-variable]" 하나, 새니타이저 보고 없음, 종료 상태 0):
//   value: 7
//   value: 7
//   count: 1
//   count: 0

#include <stdio.h>

#define PRINT_TWICE(label, x)                  \
    do {                                       \
        int count = 2;                         \
        while (count-- > 0) {                  \
            printf("%s: %d\n", (label), (x)); \
        }                                      \
    } while (0)

int main(void) {
    int value = 7;
    PRINT_TWICE("value", value); // 기대대로 7을 두 번

    int count = 7;
    PRINT_TWICE("count", count); // 매크로의 count를 찍음
    return 0;
}
