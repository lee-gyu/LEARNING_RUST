// 부록 B ch12 · Q12.3 — 새니타이저가 보고하지 않는 반복자 무효화
//
// ch9의 compare/06과 같은 코드(범위 기반 for로 돌면서 push_back)에 reserve 한 줄만 더했습니다. 용량이 넉넉해
// 재할당이 일어나지 않으므로 해제된 메모리에 닿지 않고, ASan·UBSan은 아무것도 보고하지 않습니다.
// 그래도 C++ 표준 초안([vector.modifiers])은 재할당이 없어도 끝 반복자를 무효로 보므로, 범위 기반 for가 잡아 둔
// 끝 반복자와 비교하는 이 코드는 미정의 동작입니다. 새니타이저는 "실행된 경로에서 실제로 잘못된 메모리에
// 닿은 것"을 잡을 뿐, 표준이 금지한 코드를 모두 잡지는 않습니다.
// libc++ 하드닝(-D_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_DEBUG)을 켜도 보고하지 않았고(종료 0),
// clang-tidy(bugprone-*, clang-analyzer-*, cppcoreguidelines-*)도 반복자 무효화를 보고하지 않았습니다.
// 미정의 동작이므로 일반 실행의 출력은 싣지 않습니다.
// Rust에서 같은 구조는 용량과 상관없이 E0502입니다(Part 0 ch8 4.1절, ch9 Q9.7).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch12-modern-cpp-enough
//   c++ -std=c++20 -g -fsanitize=address,undefined -o target/tmp/faq-ch12-modern-cpp-enough/reserve_hides_bug appd-b/ch12-modern-cpp-enough/compare/02_reserve_hides_bug.cpp
//   target/tmp/faq-ch12-modern-cpp-enough/reserve_hides_bug > /dev/null; echo $?
//
// 기대하는 결과: 새니타이저 보고 없음, 종료 상태 0.

#include <cstdio>
#include <vector>

int main() {
    std::vector<int> v{1, 2, 3};
    v.reserve(8); // 재할당이 일어나지 않을 만큼 용량을 미리 잡음
    for (int &x : v) {
        if (x == 2) {
            v.push_back(4); // 끝 반복자가 무효가 됨
        }
    }
    std::printf("%zu\n", v.size());
}
