// 부록 B ch8 · Q8.2 · Q8.3 · Q8.4 — 잠금을 빠뜨린 스레드 하나가 만드는 데이터 경쟁
//
// 두 스레드가 전역 카운터를 10만 번씩 1 올립니다. 잠금(pthread_mutex_t)은 카운터와 따로 있어서,
// 한 스레드는 잠그고 고치지만 다른 스레드는 잠그는 것을 빠뜨렸습니다. C 컴파일러는 이것을 막지 않고,
// 마지막 합계는 실행할 때마다 달라질 수 있습니다. ThreadSanitizer(TSan)를 켜고 실행하면 경쟁이 실제로
// 일어난 실행에서 보고합니다.
//
// Rust에서는 Mutex가 데이터를 소유하므로 잠그지 않고는 카운터에 닿을 수 없습니다
// (compile_fail/e0368_mutex_used_as_value.rs).
//
// 컴파일·실행(저장소 루트에서, Apple clang 21.0.0 / macOS에서 확인). cc의 -o는 디렉터리를 만들지 않으므로 먼저 만듭니다:
//   mkdir -p target/tmp/faq-ch8-concurrency-async
//   cc -g -fsanitize=thread -o target/tmp/faq-ch8-concurrency-async/data_race appd-b/ch8-concurrency-async/compare/01_data_race_counter.c
//   target/tmp/faq-ch8-concurrency-async/data_race
//
// 기대하는 결과: "WARNING: ThreadSanitizer: data race"를 보고합니다. 합계 줄은 실행마다 다를 수 있어
// README에 싣지 않습니다. TSan은 보고 뒤에도 끝까지 실행하고(halt_on_error=false), 이 환경에서는
// abort_on_error가 켜져 있어(TSAN_OPTIONS=help=1로 확인) 프로그램이 끝날 때 abort()하므로 종료 상태는 134입니다.

#include <pthread.h>
#include <stdio.h>

static long counter = 0;
static pthread_mutex_t counter_lock = PTHREAD_MUTEX_INITIALIZER;

static void *add_locked(void *arg) {
    (void)arg;
    for (int i = 0; i < 100000; i++) {
        pthread_mutex_lock(&counter_lock);
        counter++;
        pthread_mutex_unlock(&counter_lock);
    }
    return NULL;
}

static void *add_forgot_lock(void *arg) {
    (void)arg;
    for (int i = 0; i < 100000; i++) {
        counter++; // 잠그는 것을 빠뜨림: 컴파일러는 알려 주지 않음
    }
    return NULL;
}

int main(void) {
    pthread_t a, b;
    pthread_create(&a, NULL, add_locked, NULL);
    pthread_create(&b, NULL, add_forgot_lock, NULL);
    pthread_join(a, NULL);
    pthread_join(b, NULL);
    printf("counter = %ld (기대값 200000)\n", counter);
    return 0;
}
