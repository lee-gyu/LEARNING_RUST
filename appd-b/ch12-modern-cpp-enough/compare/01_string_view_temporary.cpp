// 부록 B ch12 · Q12.2 — 함수를 거친 string_view가 임시 string을 가리킴
//
// make_line()이 돌려준 임시 std::string은 이 문장이 끝나면 소멸합니다. first_word는 받은 string_view의 일부를
// 돌려주므로 w는 소멸한 문자열의 버퍼를 가리킵니다. 문자열이 짧으면 객체 안의 버퍼(SSO)에 담기므로, 힙 버퍼를
// 쓰도록 길게 만들었습니다.
// - c++ -std=c++20 -Wall -Wextra: 경고 없음. 같은 일을 함수 없이 한 줄(std::string_view w = make_line();)로 쓰면
//   clang이 -Wdangling-gsl로 경고하지만, 함수를 한 번 거치면 경고하지 않습니다. first_word의 매개변수에
//   [[clang::lifetimebound]]를 붙이면 그때는 경고합니다(이 환경에서 확인).
// - clang-tidy(bugprone-*, clang-analyzer-*, cppcoreguidelines-*): 이 파일의 수명 문제는 보고하지 않음.
//   한 줄로 쓴 경우는 bugprone-dangling-handle이 보고합니다.
// Rust에서 같은 구조(임시 String을 빌린 &str을 계속 씀)는 E0716입니다(Part 1 ch3 4.2절).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch12-modern-cpp-enough
//   c++ -std=c++20 -g -fsanitize=address,undefined -o target/tmp/faq-ch12-modern-cpp-enough/string_view_temporary appd-b/ch12-modern-cpp-enough/compare/01_string_view_temporary.cpp
//   target/tmp/faq-ch12-modern-cpp-enough/string_view_temporary
//
// 기대하는 결과: "ERROR: AddressSanitizer: heap-use-after-free", 종료 상태 134.

#include <cstdio>
#include <string>
#include <string_view>

std::string make_line() {
    return std::string("request-handler started with a fairly long configuration line");
}

std::string_view first_word(std::string_view s) { return s.substr(0, s.find(' ')); }

int main() {
    std::string_view w = first_word(make_line()); // 임시 string은 이 문장 끝에서 소멸
    std::printf("%.*s\n", static_cast<int>(w.size()), w.data());
}
