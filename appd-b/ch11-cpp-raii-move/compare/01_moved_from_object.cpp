// 부록 B ch11 · Q11.2 — std::move 뒤에도 남는 원본 객체
//
// Buffer는 이동 생성자에서 원본의 이름을 비워 두는 클래스입니다. b = std::move(a) 뒤에도 a는 살아 있는 객체라서
// a.name()을 부를 수 있고(이 클래스는 빈 문자열을 돌려주도록 정했음), 함수가 끝나면 a의 소멸자도 실행됩니다.
// 그래서 소멸자가 객체 수(2)만큼 불립니다. 표준 라이브러리 타입이라면 이동된 객체는 "유효하지만 지정되지 않은
// 상태"라서 a.name()이 무엇을 돌려줄지 단정할 수 없지만, 여기서는 이동 생성자를 직접 정의했으므로 출력이 정해져
// 있습니다. clang은 이동 뒤 사용을 경고하지 않습니다(-Wall -Wextra). Rust판은 examples/01_moves_and_drops.rs의 1번입니다.
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). c++의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch11-cpp-raii-move
//   c++ -std=c++20 -Wall -Wextra -o target/tmp/faq-ch11-cpp-raii-move/moved_from_object appd-b/ch11-cpp-raii-move/compare/01_moved_from_object.cpp
//   target/tmp/faq-ch11-cpp-raii-move/moved_from_object
//
// 기대하는 결과(경고 없음, 출력은 정해져 있음):
//   생성: 요청 버퍼
//   이동: 요청 버퍼
//   이동 뒤 a.name(): ""
//   소멸: 요청 버퍼
//   소멸: ""

#include <cstdio>
#include <string>
#include <utility>

class Buffer {
public:
    explicit Buffer(std::string name) : name_(std::move(name)) {
        std::printf("생성: %s\n", name_.c_str());
    }
    Buffer(Buffer &&other) noexcept : name_(std::move(other.name_)) {
        other.name_.clear(); // 원본을 빈 상태로 남김
        std::printf("이동: %s\n", name_.c_str());
    }
    ~Buffer() {
        std::printf("소멸: %s\n", name_.empty() ? "\"\"" : name_.c_str());
    }
    const std::string &name() const { return name_; }

private:
    std::string name_;
};

int main() {
    Buffer a("요청 버퍼");
    Buffer b = std::move(a);
    std::printf("이동 뒤 a.name(): \"%s\"\n", a.name().c_str()); // 이동한 뒤에도 쓸 수 있음
}
