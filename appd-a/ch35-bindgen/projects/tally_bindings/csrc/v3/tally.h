/*
 * tally.h - 정수 표본을 구간별로 세는 작은 히스토그램 라이브러리 (v3)
 *
 * v1에서 바뀐 곳에 [v2], v2에서 바뀐 곳에 [v3] 표시를 달았습니다. v3에서는 tally_record의
 * 인자가 바뀌었으므로, 이 헤더로 다시 컴파일하는 C 프로그램은 C 컴파일러가 인자 수가
 * 맞지 않는 호출을 에러로 알려 줍니다.
 *
 * 이 라이브러리의 약속
 * - 공개 이름은 모두 tally_ 또는 TALLY_로 시작합니다. 링커가 보는 이름 공간은
 *   프로그램 전체에 하나뿐이라서, 접두사로 다른 라이브러리와 겹치지 않게 합니다.
 * - 경계를 넘는 정수는 <stdint.h>의 크기가 정해진 타입만 씁니다.
 * - 할당과 해제는 모두 이 라이브러리 안에서 합니다. tally_new가 만든 것은
 *   tally_free로, tally_render가 돌려준 문자열은 tally_string_free로 해제합니다.
 * - 스레드: 전역 상태가 없으므로 서로 다른 tally는 여러 스레드에서 동시에 써도
 *   됩니다. tally 하나를 다른 스레드로 넘겨 써도 됩니다. 그러나 같은 tally를
 *   여러 스레드에서 동시에 부르면 안 됩니다(읽기 함수끼리도 마찬가지입니다).
 */
#ifndef TALLY_H
#define TALLY_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* 구간 수의 상한 */
#define TALLY_MAX_BUCKETS 32
/* 이름의 최대 바이트 수(끝의 0 바이트 제외). [v2] 31에서 15로 줄었습니다. */
#define TALLY_NAME_MAX 15

/*
 * 상태 코드. 함수는 이 값을 int32_t로 돌려줍니다.
 * C enum 타입 자체의 크기는 컴파일러가 정하므로 함수 시그니처에는 쓰지 않습니다.
 */
enum tally_status {
    TALLY_OK = 0,
    TALLY_ERR_NULL = -1,
    TALLY_ERR_EMPTY_NAME = -2,   /* [v2] 새 상태 코드 */
    TALLY_ERR_NAME_TOO_LONG = -3 /* [v2] -2에서 -3으로 */
};

/*
 * 값으로 주고받는 설정. 구간 i는 [low + i * bucket_width, low + (i + 1) * bucket_width)입니다.
 * bucket_width는 1 이상, bucket_count는 1 이상 TALLY_MAX_BUCKETS 이하여야 하고,
 * 마지막 구간의 끝이 int64_t 범위를 넘으면 안 됩니다.
 */
typedef struct tally_config {
    int64_t low;
    uint32_t bucket_width;
    uint32_t bucket_count;
} tally_config;

/* 값으로 돌려받는 요약 */
typedef struct tally_summary {
    uint64_t count; /* 기록한 표본 수(범위 밖 포함) */
    uint64_t below; /* 첫 구간보다 작은 표본 수 */
    uint64_t above; /* 마지막 구간 이상인 표본 수 */
    int64_t max;    /* count가 0이면 0. [v2] max를 min 앞으로 옮겼습니다. */
    int64_t min;    /* count가 0이면 0 */
} tally_summary;

/* 불투명 타입: 필드 정의는 tally.c 안에만 있습니다. */
typedef struct tally tally;

/* 기본 설정: low 0, 너비 10, 구간 10개 */
tally_config tally_default_config(void);

/* 새 tally를 만듭니다. 설정이 잘못됐거나 메모리가 없으면 NULL을 돌려줍니다. */
tally *tally_new(tally_config config);

/* tally를 해제합니다. NULL을 넘겨도 됩니다. 해제한 뒤에는 t를 쓰면 안 됩니다. */
void tally_free(tally *t);

/*
 * 표본 하나를 weight번 기록합니다. TALLY_OK 또는 TALLY_ERR_NULL
 * [v3] weight 인자가 더해졌습니다(v2까지는 tally_record(t, sample)).
 */
int32_t tally_record(tally *t, int64_t sample, uint32_t weight);

/* 요약을 값으로 돌려줍니다. t가 NULL이면 모든 필드가 0입니다. */
tally_summary tally_summarize(const tally *t);

/*
 * 이름을 복사해 둡니다. name은 0 바이트로 끝나야 합니다.
 * TALLY_OK, TALLY_ERR_NULL, TALLY_ERR_EMPTY_NAME([v2] 빈 이름), TALLY_ERR_NAME_TOO_LONG
 */
int32_t tally_set_name(tally *t, const char *name);

/*
 * 빌려주는 문자열: t 안의 버퍼를 가리킵니다. 다음 tally_set_name이나 tally_free를
 * 부를 때까지만 유효하고, 호출자가 해제하면 안 됩니다. t가 NULL이면 NULL입니다.
 */
const char *tally_name(const tally *t);

/*
 * 넘겨주는 문자열: 새로 할당한 표를 돌려줍니다. 호출자는 다 쓴 뒤
 * tally_string_free로 해제해야 합니다. t가 NULL이거나 메모리가 없으면 NULL입니다.
 */
char *tally_render(const tally *t);

/* tally_render가 돌려준 문자열을 해제합니다. NULL을 넘겨도 됩니다. */
void tally_string_free(char *s);

#ifdef __cplusplus
}
#endif

#endif /* TALLY_H */
