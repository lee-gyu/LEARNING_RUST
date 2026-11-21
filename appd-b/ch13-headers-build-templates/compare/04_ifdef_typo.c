// 부록 B ch13 · Q13.10 — #ifdef의 이름을 잘못 적어도 아무도 알려 주지 않음
//
// macOS에서 clang은 __APPLE__을 정의합니다. 여기서는 일부러 __APLE__로 잘못 적었습니다. 정의되지 않은 이름이므로
// #ifdef는 거짓이 되고, macOS에서도 "그 밖의 플랫폼" 쪽이 컴파일됩니다. 전처리기는 이 이름이 어딘가에서
// 정의될 수도 있다고 보므로 -Wall -Wextra, 그리고 #if에서 정의되지 않은 이름을 알리는 -Wundef를 켜도 경고가 없습니다.
// 출력이 정해져 있습니다.
// Rust에서는 Cargo가 컴파일러에 알려진 cfg 이름과 값을 넘기므로, Cargo로 빌드할 때 target_os = "macoss" 같은
// 오타는 unexpected_cfgs 경고가 됩니다(기능 이름의 오타는 부록 A ch26 예제 03).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch13-headers-build-templates
//   cc -std=c17 -Wall -Wextra -Wundef -g -fsanitize=address,undefined -o target/tmp/faq-ch13-headers-build-templates/ifdef_typo appd-b/ch13-headers-build-templates/compare/04_ifdef_typo.c
//   target/tmp/faq-ch13-headers-build-templates/ifdef_typo
//
// 기대하는 결과(경고·새니타이저 보고 없음, 종료 상태 0, macOS에서):
//   그 밖의 플랫폼 경로

#include <stdio.h>

int main(void) {
#ifdef __APLE__ // __APPLE__의 오타
    puts("macOS 경로");
#else
    puts("그 밖의 플랫폼 경로");
#endif
    return 0;
}
