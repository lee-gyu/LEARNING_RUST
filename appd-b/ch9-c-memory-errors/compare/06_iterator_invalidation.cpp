// 부록 B ch9 · Q9.7 — 반복자 무효화
//
// 범위 기반 for로 vector를 돌면서 push_back합니다. 용량이 모자라면 원소가 새 공간으로 옮겨지고, 순회 중인
// 반복자와 참조 x는 해제된 옛 공간을 가리킵니다. C++ 표준 초안([vector.modifiers])은 재할당이 원소를
// 가리키던 모든 참조·포인터·반복자를 무효로 만들고, 재할당이 없어도 넣은 자리 뒤와 끝 반복자는 무효라고
// 정합니다. 범위 기반 for는 시작할 때 잡아 둔 끝 반복자와 비교하므로, 용량과 상관없이 미정의 동작입니다.
// Rust에서 같은 모양은 E0502로 컴파일되지 않습니다(Part 0 ch8 4.1절). 해법은 examples/03_modify_while_iterating.rs.
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch9-c-memory-errors
//   c++ -std=c++20 -g -fsanitize=address,undefined -o target/tmp/faq-ch9-c-memory-errors/iterator_invalidation appd-b/ch9-c-memory-errors/compare/06_iterator_invalidation.cpp
//   target/tmp/faq-ch9-c-memory-errors/iterator_invalidation
//
// 기대하는 결과: "ERROR: AddressSanitizer: heap-use-after-free", 종료 상태 134.

#include <cstdio>
#include <vector>

int main() {
    std::vector<int> v{1, 2, 3};
    for (int &x : v) {
        if (x == 2) {
            v.push_back(4); // 순회 중에 재할당이 일어날 수 있음
        }
    }
    std::printf("%zu\n", v.size());
}
