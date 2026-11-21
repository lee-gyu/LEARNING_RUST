/* semver.c - 원래의 C 구현. Rust 구현으로 바꿔 끼울 대상입니다. */
#include "semver.h"

#include <stddef.h>

int semver_parse(const char *text, semver_t *out) {
    uint32_t parts[3];
    int count = 0;
    const char *p = text;

    if (text == NULL || out == NULL) {
        return -1;
    }
    for (;;) {
        uint32_t value = 0;
        const char *start = p;
        while (*p >= '0' && *p <= '9') {
            /* 오버플로를 검사하지 않습니다. 부호 없는 정수라 미정의 동작은 아니고 2^32로 나눈 나머지가 됩니다. */
            value = value * 10u + (uint32_t)(*p - '0');
            p++;
        }
        if (p == start || count == 3) {
            return -1;
        }
        parts[count++] = value;
        if (*p == '.') {
            p++;
            continue;
        }
        break;
    }
    if (*p != '\0' || count != 3) {
        return -1;
    }
    out->major = parts[0];
    out->minor = parts[1];
    out->patch = parts[2];
    return 0;
}
