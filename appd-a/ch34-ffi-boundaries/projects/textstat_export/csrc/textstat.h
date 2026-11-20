/*
 * textstat.h - Rust로 구현한 UTF-8 텍스트 통계 라이브러리의 C 인터페이스
 *
 * 구현은 src/textstat.rs에 있고, 이 헤더는 그 선언을 손으로 옮긴 것입니다.
 * - 이름은 모두 textstat_ 또는 TEXTSTAT_로 시작합니다.
 * - textstat_counter_new가 돌려준 핸들은 textstat_counter_free로만 해제합니다
 *   (Rust가 할당했으므로 Rust가 해제합니다. C의 free()를 쓰면 안 됩니다).
 * - 문자열을 돌려받을 때는 호출자가 버퍼를 줍니다(할당과 해제 모두 호출자 쪽).
 * - 함수는 패닉을 밖으로 내보내지 않습니다. Rust 안에서 패닉이 나면 TEXTSTAT_ERR_PANIC입니다.
 * - 한 핸들을 여러 스레드에서 동시에 쓰면 안 됩니다.
 */
#ifndef TEXTSTAT_H
#define TEXTSTAT_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define TEXTSTAT_OK 0
#define TEXTSTAT_ERR_NULL (-1)
#define TEXTSTAT_ERR_UTF8 (-2)
#define TEXTSTAT_ERR_PANIC (-99)

typedef struct textstat_counts {
    uint64_t bytes;
    uint64_t chars;
    uint64_t words;
    uint64_t lines;
} textstat_counts;

/* 불투명 타입: 실제로는 Rust의 Counter입니다. */
typedef struct textstat_counter textstat_counter;

uint32_t textstat_version(void);

/*
 * text 전체를 셉니다. TEXTSTAT_OK, TEXTSTAT_ERR_NULL, TEXTSTAT_ERR_UTF8
 * out은 결과를 쓰기만 하는 출력 매개변수라서 초기화하지 않은 변수의 주소를 넘겨도 됩니다.
 * TEXTSTAT_OK일 때만 *out에 값을 씁니다.
 */
int32_t textstat_count(const char *text, textstat_counts *out);

/* 조각으로 나눠 세는 핸들. finish의 out도 위와 같은 출력 매개변수입니다. */
textstat_counter *textstat_counter_new(void);
int32_t textstat_counter_feed(textstat_counter *counter, const char *chunk);
int32_t textstat_counter_finish(textstat_counter *counter, textstat_counts *out);
void textstat_counter_free(textstat_counter *counter);

/*
 * counts를 문장으로 만들어 buf에 씁니다. counts는 초기화된 값을 가리켜야 합니다(읽습니다).
 * snprintf처럼 필요한 길이(끝의 0 바이트 제외)를 돌려주고, buf가 모자라면 내용을 쓰지
 * 않습니다(buf_len이 0보다 크면 빈 문자열). buf는 초기화돼 있지 않아도 됩니다.
 */
int32_t textstat_describe(const textstat_counts *counts, char *buf, size_t buf_len);

#ifdef __cplusplus
}
#endif

#endif /* TEXTSTAT_H */
