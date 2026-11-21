// 부록 B ch17 · Q17.3 — C++ 함수의 링크 이름과 extern "C" 함수의 링크 이름
//
// C++는 오버로딩과 네임스페이스를 위해 함수 이름에 매개변수 타입 등을 덧붙인 링크 이름(맹글링된 이름)을 씁니다.
// Rust의 extern "C" 블록은 C의 이름으로 함수를 찾으므로, C++ 함수를 부르려면 extern "C"로 감싼 함수를 둡니다.
// 미정의 동작은 없습니다. 새니타이저를 켜고 빌드해 그대로 실행되는 것만 확인하고, 이름은 nm으로 봅니다.
//
// 확인:
//   mkdir -p target/tmp/faq-ch17-interop-migration
//   c++ -std=c++20 -fsanitize=address,undefined -o target/tmp/faq-ch17-interop-migration/cpp_mangling \
//       appd-b/ch17-interop-migration/compare/01_cpp_mangling.cpp
//   target/tmp/faq-ch17-interop-migration/cpp_mangling
//   nm target/tmp/faq-ch17-interop-migration/cpp_mangling | grep scale

#include <cstdio>

int scale(int x) { return x * 2; }
double scale(double x) { return x * 2.0; }

extern "C" int scale_c(int x) { return scale(x); }

int main() {
    std::printf("scale(2) = %d, scale(1.5) = %.1f, scale_c(3) = %d\n", scale(2), scale(1.5), scale_c(3));
    return 0;
}
