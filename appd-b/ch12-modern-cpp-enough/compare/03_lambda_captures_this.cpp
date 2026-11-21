// 부록 B ch12 · Q12.2 — [this]를 캡처한 콜백이 객체보다 오래 삶
//
// Counter::handler()는 [this]를 캡처한 람다를 돌려주고, Button은 그것을 std::function으로 보관합니다.
// counter는 unique_ptr이라 블록이 끝나면 Counter가 해제되지만, Button이 든 람다는 여전히 그 주소를 가리킵니다.
// std::function은 람다가 무엇을 가리키는지 타입에 남기지 않으므로, 컴파일러는 이 등록을 막을 근거가 없습니다.
// - c++ -std=c++20 -Wall -Wextra: 경고 없음.
// - clang-tidy(bugprone-*, clang-analyzer-*, cppcoreguidelines-*): 수명 문제를 보고하지 않음.
// Rust에서 같은 구조는 compile_fail/e0597_callback_borrows_local.rs(E0597)이고, 컴파일되는 방법은
// examples/01_callbacks_own_state.rs에 있습니다.
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch12-modern-cpp-enough
//   c++ -std=c++20 -g -fsanitize=address,undefined -o target/tmp/faq-ch12-modern-cpp-enough/lambda_captures_this appd-b/ch12-modern-cpp-enough/compare/03_lambda_captures_this.cpp
//   target/tmp/faq-ch12-modern-cpp-enough/lambda_captures_this
//
// 기대하는 결과: "ERROR: AddressSanitizer: heap-use-after-free", 종료 상태 134.

#include <cstdio>
#include <functional>
#include <memory>
#include <vector>

struct Counter {
    int clicks = 0;
    std::function<void()> handler() {
        return [this] {
            ++clicks;
            std::printf("clicks: %d\n", clicks);
        };
    }
};

struct Button {
    std::vector<std::function<void()>> handlers;
    void on_click(std::function<void()> f) { handlers.push_back(std::move(f)); }
    void click() {
        for (auto &f : handlers) {
            f();
        }
    }
};

int main() {
    Button button;
    {
        auto counter = std::make_unique<Counter>();
        button.on_click(counter->handler());
    } // Counter가 여기서 해제됨
    button.click(); // 해제된 Counter의 clicks를 고침
}
