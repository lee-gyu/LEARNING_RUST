/* main.c - 표준 입력의 줄마다 semver_parse를 부르고 결과를 출력하는 C 프로그램.
   이 파일은 어느 구현과 링크하든 그대로입니다. */
#include <stdio.h>
#include <string.h>

#include "semver.h"

int main(void) {
    char line[256];

    while (fgets(line, sizeof line, stdin) != NULL) {
        semver_t v;
        line[strcspn(line, "\n")] = '\0';
        if (semver_parse(line, &v) == 0) {
            printf("\"%s\" -> %u.%u.%u\n", line, v.major, v.minor, v.patch);
        } else {
            printf("\"%s\" -> 형식 오류\n", line);
        }
    }
    return 0;
}
