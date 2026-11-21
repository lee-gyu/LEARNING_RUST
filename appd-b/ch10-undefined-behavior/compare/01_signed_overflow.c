// 부록 B ch10 · Q10.1 · Q10.3 — 부호 있는 정수의 오버플로
//
// 다음 아이디를 id + 1로 구하는데, id가 INT_MAX입니다. C에서 부호 있는 정수의 오버플로는 미정의 동작이라
// (UndefinedBehaviorSanitizer 문서도 이것을 검사 대상으로 둠), 컴파일러는 "오버플로는 일어나지 않는다"고 가정하고
// 코드를 바꿔도 됩니다. 반면 부호 없는 정수의 오버플로는 정의된 래핑입니다.
// Rust에서 정수 오버플로는 미정의 동작이 아니라 디버그 빌드는 패닉, 릴리스 빌드는 래핑입니다(ch3 Q3.6).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch10-undefined-behavior
//   cc -g -fsanitize=undefined -o target/tmp/faq-ch10-undefined-behavior/signed_overflow appd-b/ch10-undefined-behavior/compare/01_signed_overflow.c
//   target/tmp/faq-ch10-undefined-behavior/signed_overflow
//
// 기대하는 결과: "runtime error: signed integer overflow: 2147483647 + 1 cannot be represented in type 'int'".
// UBSan은 보고한 뒤 실행을 계속하므로 종료 상태는 0입니다. 출력되는 값은 미정의 동작의 결과라 README에 싣지 않습니다.

#include <limits.h>
#include <stdio.h>

static int next_id(int id) {
    return id + 1; // id가 INT_MAX면 오버플로
}

int main(void) {
    printf("%d\n", next_id(INT_MAX));
    return 0;
}
