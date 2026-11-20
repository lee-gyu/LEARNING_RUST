/*
 * driver.c - textstat 라이브러리를 쓰는 C 프로그램
 *
 * 실제 C 프로그램이라면 이 파일에 main이 있고, Rust 쪽을 정적 라이브러리로 빌드해
 * 함께 링크합니다. 이 프로젝트는 cargo만으로 확인하려고 main 대신 c_program_main을
 * 두고, Rust의 main이 이 함수 하나만 부릅니다. 그 뒤로는 C가 textstat_* 함수를 부릅니다.
 */
#include <inttypes.h>
#include <stdio.h>

#include "textstat.h"

static void show(const char *label, int32_t status, const textstat_counts *c) {
    if (status == TEXTSTAT_OK) {
        printf("%s -> bytes=%" PRIu64 " chars=%" PRIu64 " words=%" PRIu64 " lines=%" PRIu64 "\n",
               label, c->bytes, c->chars, c->words, c->lines);
    } else {
        printf("%s -> 에러 %" PRId32 "\n", label, status);
    }
}

int c_program_main(void) {
    textstat_counts c = {0, 0, 0, 0};
    printf("[C] textstat_version() = %" PRIu32 "\n", textstat_version());

    printf("[C] 1. 텍스트 전체를 한 번에 세기\n");
    show("    \"hello, 러스트\\n둘째 줄\"", textstat_count("hello, 러스트\n둘째 줄", &c), &c);

    printf("[C] 2. 잘못된 인자\n");
    show("    text = NULL", textstat_count(NULL, &c), &c);
    show("    out = NULL", textstat_count("abc", NULL), &c);
    show("    잘못된 UTF-8", textstat_count("\xff\xfe", &c), &c);

    printf("[C] 3. Rust가 만든 핸들에 조각으로 나눠 넣기\n");
    textstat_counter *k = textstat_counter_new();
    /* "녕"의 UTF-8 바이트(EB 85 95)가 둘째와 셋째 조각에 걸쳐 있습니다. */
    int32_t s1 = textstat_counter_feed(k, "안");
    int32_t s2 = textstat_counter_feed(k, "\xeb\x85");
    int32_t s3 = textstat_counter_feed(k, "\x95 world\n");
    printf("    feed 결과: %" PRId32 " %" PRId32 " %" PRId32 "\n", s1, s2, s3);
    show("    finish", textstat_counter_finish(k, &c), &c);

    printf("[C] 4. 사용 규칙 위반: finish 뒤에 feed\n");
    int32_t late = textstat_counter_feed(k, "more");
    printf("    feed 결과: %" PRId32 " (TEXTSTAT_ERR_PANIC = %d)\n", late, TEXTSTAT_ERR_PANIC);
    textstat_counter_free(k);    /* Rust가 할당한 것은 Rust의 해제 함수로 */
    textstat_counter_free(NULL); /* NULL은 아무 일도 하지 않습니다 */

    printf("[C] 5. 문자열 받기: C의 버퍼에 Rust가 씀\n");
    char buf[64];
    int32_t n = textstat_describe(&c, buf, sizeof buf);
    printf("    필요한 길이 %" PRId32 ", 버퍼 %zu바이트: \"%s\"\n", n, sizeof buf, buf);
    char tiny[8];
    n = textstat_describe(&c, tiny, sizeof tiny);
    printf("    필요한 길이 %" PRId32 ", 버퍼 %zu바이트: \"%s\"\n", n, sizeof tiny, tiny);

    /* C의 stdout 버퍼는 Rust의 것과 따로 있습니다. 돌아가기 전에 비워 출력 순서를 지킵니다. */
    fflush(stdout);
    return 0;
}
