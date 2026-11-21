// 부록 B ch13 · Q13.5 — 템플릿 본문은 쓰는 자리에서 실제 타입으로 검사됨
//
// total_size는 원소의 size()를 더하는 템플릿입니다. 제약이 없으므로 본문의 t.size()는 정의할 때가 아니라
// 실제 타입으로 쓰일 때(인스턴스화) 검사됩니다. std::string으로만 쓰면 문제없이 컴파일되고,
// int로 쓰는 줄(-DUSE_INT)을 더해야 비로소 에러가 납니다. 그 에러는 템플릿 본문 안을 가리키고,
// 어디서 쓰였는지는 note로 따라 나옵니다.
// C++20 concept(HasSize)으로 제약을 적은 total_size_checked는 int로 부르면(-DUSE_INT_CHECKED) 부른 자리에서
// "constraints not satisfied"로 거부되고, 어떤 요구를 못 채웠는지 note가 이어집니다.
// Rust의 제네릭은 본문을 정의할 때 트레이트 바운드만으로 검사합니다. 바운드 없는 T로 연산자나 메서드를 쓰면
// 아무도 부르지 않아도 에러이고(연산자는 Part 0 ch10 4.1절의 E0369, 메서드는 E0599), 바운드를 못 채운 타입으로
// 부르면 부른 자리에서 E0277입니다(Part 0 ch10 4.2절).
//
// 저장소 루트에서 실행합니다(Apple clang 21.0.0 / macOS에서 확인).
//   c++ -std=c++20 -Wall -Wextra -fsyntax-only appd-b/ch13-headers-build-templates/compare/03_template_checked_late.cpp
//     → 경고·에러 없음
//   c++ -std=c++20 -Wall -Wextra -fsyntax-only -DUSE_INT appd-b/ch13-headers-build-templates/compare/03_template_checked_late.cpp
//     → error: member reference base type 'const int' is not a structure or union (본문의 t.size() 줄)
//       note: in instantiation of function template specialization 'total_size<int>' requested here
//   c++ -std=c++20 -Wall -Wextra -fsyntax-only -DUSE_INT_CHECKED appd-b/ch13-headers-build-templates/compare/03_template_checked_late.cpp
//     → error: no matching function for call to 'total_size_checked' (부른 줄)
//       note: candidate template ignored: constraints not satisfied [with T = int]
//       note: because 'int' does not satisfy 'HasSize'

#include <concepts>
#include <cstddef>
#include <cstdio>
#include <string>
#include <vector>

// 제약 없는 템플릿
template <typename T>
std::size_t total_size(const std::vector<T> &items) {
    std::size_t sum = 0;
    for (const auto &t : items) {
        sum += t.size(); // T가 무엇인지 정해질 때 검사됨
    }
    return sum;
}

// C++20 concept으로 제약을 적은 판
template <typename T>
concept HasSize = requires(const T &t) {
    { t.size() } -> std::convertible_to<std::size_t>;
};

template <HasSize T>
std::size_t total_size_checked(const std::vector<T> &items) {
    std::size_t sum = 0;
    for (const auto &t : items) {
        sum += t.size();
    }
    return sum;
}

int main() {
    std::vector<std::string> words{"ab", "cde"};
    std::printf("%zu %zu\n", total_size(words), total_size_checked(words));
#ifdef USE_INT
    std::vector<int> numbers{1, 2, 3};
    std::printf("%zu\n", total_size(numbers));
#endif
#ifdef USE_INT_CHECKED
    std::vector<int> numbers{1, 2, 3};
    std::printf("%zu\n", total_size_checked(numbers));
#endif
}
