// 부록 B ch13 · Q13.6 — ODR 위반: 같은 inline 함수를 두 파일에서 다르게 정의
//
// 이 파일과 05_odr_other.cpp는 둘 다 inline int limit()을 정의하지만 돌려주는 값이 다릅니다. inline 함수는
// 여러 번역 단위에 정의가 있어도 되는 대신 모든 정의가 같아야 하고(단일 정의 규칙, ODR), 다르면 프로그램이
// 잘못된 것이지만 진단은 요구되지 않습니다(표준 초안 [basic.def.odr]). 링커는 둘 중 하나를 골라 쓰고 알리지 않습니다.
// 이 환경에서 아래 명령은 경고 없이 링크되었고, ASan·UBSan도(ASAN_OPTIONS=detect_odr_violation=2를 주어도) 보고하지
// 않았으며, -flto -O2로 링크해도 진단이 없었습니다. 잘못된 프로그램이므로 실행 출력은 싣지 않습니다.
// Rust에서 같은 이름의 함수는 모듈 경로가 달라 서로 다른 항목입니다. 심볼 이름을 그대로 내보내는
// #[unsafe(no_mangle)] 함수 둘이 같은 크레이트 안에서 같은 이름이면 "symbol `answer` is already defined"로
// 컴파일되지 않습니다(compile_fail/no_mangle_symbol_clash.rs). 서로 다른 크레이트 사이라면 링커 경고만 나고
// 한쪽 정의가 쓰입니다(README Q13.6).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch13-headers-build-templates
//   c++ -std=c++20 -Wall -Wextra -g -fsanitize=address,undefined -o target/tmp/faq-ch13-headers-build-templates/odr appd-b/ch13-headers-build-templates/compare/05_odr_main.cpp appd-b/ch13-headers-build-templates/compare/05_odr_other.cpp
//   target/tmp/faq-ch13-headers-build-templates/odr > /dev/null; echo $?
//
// 기대하는 결과: 컴파일·링크 경고 없음, 새니타이저 보고 없음, 종료 상태 0(출력은 싣지 않음).

#include <cstdio>

inline int limit() { return 10; }

int other_limit(); // 05_odr_other.cpp에 정의

int main() { std::printf("%d %d\n", limit(), other_limit()); }
