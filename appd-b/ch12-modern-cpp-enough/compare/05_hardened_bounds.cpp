// 부록 B ch12 · Q12.4 — 표준 라이브러리 하드닝이 잡는 것
//
// C++26은 표준 라이브러리 하드닝을 넣어, 하드닝된 구현에서는 vector의 operator[] 같은 함수의 전제 조건
// (인덱스 < size())을 실행 중에 검사하고 어기면 프로그램을 끝내게 했습니다. 이 환경의 libc++는 같은 일을
// _LIBCPP_HARDENING_MODE 매크로로 이미 제공합니다. 인덱스는 실행 인자 개수에서 만들어 컴파일러가 미리 알 수 없게
// 했습니다(인자 없이 실행하면 3).
// - 하드닝 없이(_LIBCPP_HARDENING_MODE_NONE): 경계 밖 읽기라 미정의 동작이므로 출력은 싣지 않습니다.
// - FAST, EXTENSIVE: 메시지 없이 멈춤(종료 상태 133, SIGTRAP).
// - DEBUG: 아래 메시지를 내고 멈춤(종료 상태 134).
// 같은 DEBUG 모드로 compare/01~04를 컴파일해 실행하면 넷 다 아무 보고 없이 종료 상태 0이었습니다.
// 하드닝은 경계 검사 같은 전제 조건을 잡지만, 수명 문제는 잡지 않습니다.
// Rust의 인덱싱은 언제나 경계를 검사하고 패닉합니다(ch9 Q9.4).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch12-modern-cpp-enough
//   c++ -std=c++20 -D_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_DEBUG -o target/tmp/faq-ch12-modern-cpp-enough/hardened_bounds appd-b/ch12-modern-cpp-enough/compare/05_hardened_bounds.cpp
//   target/tmp/faq-ch12-modern-cpp-enough/hardened_bounds
//
// 기대하는 결과: "libc++ Hardening assertion __n < size() failed: vector[] index out of bounds", 종료 상태 134.

#include <cstddef>
#include <cstdio>
#include <vector>

int main(int argc, char **) {
    std::vector<int> v{1, 2, 3};
    std::size_t i = static_cast<std::size_t>(argc) + 2; // 인자 없이 실행하면 3
    std::printf("%d\n", v[i]);
}
