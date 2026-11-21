/* semver.h - C 프로그램과 두 구현(semver.c, Rust의 src/lib.rs)이 함께 따르는 선언 */
#ifndef SEMVER_H
#define SEMVER_H

#include <stdint.h>

typedef struct {
    uint32_t major;
    uint32_t minor;
    uint32_t patch;
} semver_t;

/* "1.2.3"처럼 점으로 나눈 숫자 세 개를 읽어 out에 채웁니다.
   성공하면 0, 형식이 맞지 않으면 -1을 돌려주고 out은 건드리지 않습니다. */
int semver_parse(const char *text, semver_t *out);

#endif
