// 부록 B ch11 · Q11.4 — 소멸자만 정의하고 복사를 정의하지 않은 클래스(Rule of three 위반)
//
// Blob은 생성자에서 new[]로 메모리를 잡고 소멸자에서 delete[]합니다. 그런데 복사 생성자를 정의하지 않아서,
// 컴파일러가 만든 복사 생성자가 포인터만 복사합니다. copy와 original이 같은 메모리를 가리키다가 둘 다 소멸하면서
// 같은 메모리를 두 번 해제합니다. C++ Core Guidelines(C.21)가 "복사·이동·소멸 가운데 하나를 정의하면 모두
// 정의하거나 지우라"고 하는 이유입니다.
// Rust에서는 Drop을 구현한 타입이 저절로 복사되지 않습니다(Copy와 Drop은 함께 구현할 수 없고, Clone은 직접
// 구현하거나 derive해야 함). examples/01_moves_and_drops.rs의 3번을 보세요.
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch11-cpp-raii-move
//   c++ -std=c++20 -g -fsanitize=address,undefined -o target/tmp/faq-ch11-cpp-raii-move/rule_of_three appd-b/ch11-cpp-raii-move/compare/02_rule_of_three.cpp
//   target/tmp/faq-ch11-cpp-raii-move/rule_of_three
//
// 기대하는 결과: "ERROR: AddressSanitizer: attempting double-free", 종료 상태 134.

#include <cstddef>
#include <cstdio>

class Blob {
public:
    explicit Blob(std::size_t size) : data_(new int[size]), size_(size) {}
    ~Blob() { delete[] data_; }
    // 복사 생성자와 복사 대입 연산자를 정의하지 않음 → 컴파일러가 포인터만 복사하는 판을 만듦
    std::size_t size() const { return size_; }

private:
    int *data_;
    std::size_t size_;
};

int main() {
    Blob original(4);
    Blob copy = original; // 포인터만 복사
    std::printf("%zu %zu\n", original.size(), copy.size());
} // copy와 original의 소멸자가 같은 메모리를 해제
