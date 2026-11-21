#ifndef FLETCHER_H
#define FLETCHER_H

#include <stddef.h>
#include <stdint.h>

/* 이 라이브러리의 버전. 빌드 스크립트가 읽어 의존하는 패키지에 넘깁니다. */
#define FLETCHER_VERSION 2

/* data의 len바이트에 대한 Fletcher-16 체크섬 */
uint16_t fletcher16(const uint8_t *data, size_t len);

/* 버전 2에서 추가: 체크섬이 expected와 같으면 1, 다르면 0 */
int fletcher16_verify(const uint8_t *data, size_t len, uint16_t expected);

#endif
