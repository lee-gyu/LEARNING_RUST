// 부록 B ch14 · Q14.1 — 별칭 정보가 없으면 다시 읽어야 함
//
// add_twice는 *a에 *b를 두 번 더합니다. a와 b가 같은 곳을 가리킬 수도 있으므로, 컴파일러는 *a에 쓴 뒤 *b를
// 다시 읽어야 합니다(같은 곳이면 두 번째 *b는 이미 바뀐 값). restrict를 붙인 add_twice_restrict는 "겹치지 않는다"고
// 약속하므로 *b를 한 번 읽고 두 배로 만들어 한 번에 더합니다. Rust의 &mut i32, &i32는 이 약속을 타입이 들고
// 다니므로 restrict 없이 같은 코드가 나옵니다(examples/03_noalias.rs, README Q14.1).
// restrict를 붙였는데 실제로 겹치는 포인터를 넘기면 미정의 동작이므로, 약속을 지키는 책임은 프로그래머에게 있습니다.
//
// LLVM IR 보기(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인):
//   mkdir -p target/tmp/faq-ch14-performance
//   cc -std=c17 -Wall -Wextra -O3 -S -emit-llvm -o target/tmp/faq-ch14-performance/aliasing.ll appd-b/ch14-performance/compare/02_aliasing.c
//   grep -A8 'define.*add_twice' target/tmp/faq-ch14-performance/aliasing.ll
//
// 기대하는 결과: add_twice에는 *b를 읽는 load가 두 번, add_twice_restrict에는 한 번(그리고 shl ... 1)이 보입니다.

void add_twice(int *a, const int *b) {
    *a += *b;
    *a += *b;
}

void add_twice_restrict(int *restrict a, const int *restrict b) {
    *a += *b;
    *a += *b;
}
