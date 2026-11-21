// 부록 B ch12 · Q12.2 — unique_ptr의 get()으로 얻은 관찰용 포인터를 reset() 뒤에 씀
//
// C++ Core Guidelines R.3은 원시 포인터(T*)를 "소유하지 않는 포인터"로 쓰라고 하므로, node.get()으로 얻은 raw를
// 관찰용으로 들고 있는 것 자체는 지침에 맞습니다. 문제는 raw가 node보다 오래 쓰여도 아무도 검사하지 않는다는
// 것입니다. 지침의 Lifetime 프로필은 이런 사용을 막으려는 것이지만, 정적 분석과 라이브러리 지원으로 강제해야
// 효과가 있다고 지침 스스로 밝힙니다.
// - c++ -std=c++20 -Wall -Wextra: 경고 없음.
// - clang-tidy(bugprone-*, clang-analyzer-*, cppcoreguidelines-*): 수명 문제를 보고하지 않음.
// Rust에서 같은 구조(빌려준 Box를 drop하거나 옮긴 뒤 참조를 씀)는 E0505입니다(부록 A ch14 4.1절).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch12-modern-cpp-enough
//   c++ -std=c++20 -g -fsanitize=address,undefined -o target/tmp/faq-ch12-modern-cpp-enough/unique_ptr_get_reset appd-b/ch12-modern-cpp-enough/compare/04_unique_ptr_get_reset.cpp
//   target/tmp/faq-ch12-modern-cpp-enough/unique_ptr_get_reset
//
// 기대하는 결과: "ERROR: AddressSanitizer: heap-use-after-free", 종료 상태 134.

#include <cstdio>
#include <memory>
#include <string>

struct Node {
    std::string name;
};

int main() {
    auto node = std::make_unique<Node>(Node{"root"});
    Node *raw = node.get(); // 소유하지 않는 관찰용 포인터
    node.reset();           // 소유자가 Node를 해제
    std::printf("%s\n", raw->name.c_str());
}
