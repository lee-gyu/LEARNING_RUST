/*
 * tally.c - tally.h의 구현
 *
 * Rust 쪽에서는 이 파일의 내용을 볼 수 없습니다. Rust가 아는 것은 tally.h의 선언을
 * 손으로 옮긴 src/ffi.rs뿐이고, 링커는 함수 이름만 맞춰 봅니다.
 */
#include "tally.h"

#include <inttypes.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* 불투명 타입의 실제 정의. 헤더에는 이름만 있으므로 바깥에서는 크기도 모릅니다. */
struct tally {
    tally_config config;
    tally_summary summary;
    uint64_t buckets[TALLY_MAX_BUCKETS];
    char name[TALLY_NAME_MAX + 1];
};

tally_config tally_default_config(void) {
    tally_config c;
    c.low = 0;
    c.bucket_width = 10;
    c.bucket_count = 10;
    return c;
}

static int config_is_valid(tally_config c) {
    if (c.bucket_width == 0) {
        return 0;
    }
    if (c.bucket_count == 0 || c.bucket_count > TALLY_MAX_BUCKETS) {
        return 0;
    }
    /* 너비(32비트) x 구간 수(32 이하)는 2^37보다 작아 int64_t에 들어갑니다. */
    int64_t span = (int64_t)c.bucket_width * (int64_t)c.bucket_count;
    return c.low <= INT64_MAX - span;
}

tally *tally_new(tally_config config) {
    if (!config_is_valid(config)) {
        return NULL;
    }
    tally *t = calloc(1, sizeof *t);
    if (t == NULL) {
        return NULL;
    }
    t->config = config;
    memcpy(t->name, "tally", sizeof "tally");
    return t;
}

void tally_free(tally *t) {
    free(t);
}

int32_t tally_record(tally *t, int64_t sample) {
    if (t == NULL) {
        return TALLY_ERR_NULL;
    }
    tally_summary *s = &t->summary;
    if (s->count == 0 || sample < s->min) {
        s->min = sample;
    }
    if (s->count == 0 || sample > s->max) {
        s->max = sample;
    }
    s->count += 1;
    if (sample < t->config.low) {
        s->below += 1;
        return TALLY_OK;
    }
    /* sample >= low이므로 부호 없는 뺄셈의 결과가 정확한 거리입니다. */
    uint64_t offset = (uint64_t)sample - (uint64_t)t->config.low;
    uint64_t index = offset / t->config.bucket_width;
    if (index >= t->config.bucket_count) {
        s->above += 1;
        return TALLY_OK;
    }
    t->buckets[index] += 1;
    return TALLY_OK;
}

tally_summary tally_summarize(const tally *t) {
    tally_summary zero = {0, 0, 0, 0, 0};
    if (t == NULL) {
        return zero;
    }
    return t->summary;
}

int32_t tally_set_name(tally *t, const char *name) {
    if (t == NULL || name == NULL) {
        return TALLY_ERR_NULL;
    }
    size_t len = strlen(name);
    if (len > TALLY_NAME_MAX) {
        return TALLY_ERR_NAME_TOO_LONG;
    }
    memcpy(t->name, name, len + 1);
    return TALLY_OK;
}

const char *tally_name(const tally *t) {
    if (t == NULL) {
        return NULL;
    }
    return t->name;
}

/* 출력을 세기만 하거나(buf == NULL) 버퍼에 쓰는 작은 도우미 */
typedef struct sink {
    char *buf;
    size_t cap;
    size_t len;
} sink;

static void sink_printf(sink *s, const char *fmt, ...) {
    char *dst = NULL;
    size_t room = 0;
    if (s->buf != NULL && s->len < s->cap) {
        dst = s->buf + s->len;
        room = s->cap - s->len;
    }
    va_list ap;
    va_start(ap, fmt);
    int n = vsnprintf(dst, room, fmt, ap);
    va_end(ap);
    if (n > 0) {
        s->len += (size_t)n;
    }
}

#define TALLY_BAR_MAX 30
static const char TALLY_BAR[TALLY_BAR_MAX + 1] = "##############################";

static void render_into(const tally *t, sink *s) {
    const tally_summary *sum = &t->summary;
    sink_printf(s, "[%s] n=%" PRIu64 " below=%" PRIu64 " above=%" PRIu64 "\n", t->name,
                sum->count, sum->below, sum->above);
    for (uint32_t i = 0; i < t->config.bucket_count; i++) {
        int64_t from = t->config.low + (int64_t)i * (int64_t)t->config.bucket_width;
        int64_t to = from + (int64_t)t->config.bucket_width;
        uint64_t n = t->buckets[i];
        int bar = n > TALLY_BAR_MAX ? TALLY_BAR_MAX : (int)n;
        sink_printf(s, "  [%5" PRId64 ", %5" PRId64 ") %.*s%s %" PRIu64 "\n", from, to, bar,
                    TALLY_BAR, n > TALLY_BAR_MAX ? "+" : "", n);
    }
}

char *tally_render(const tally *t) {
    if (t == NULL) {
        return NULL;
    }
    /* 첫 번째로 길이만 재고, 그만큼 할당한 뒤 두 번째로 씁니다. */
    sink measure = {NULL, 0, 0};
    render_into(t, &measure);
    char *buf = malloc(measure.len + 1);
    if (buf == NULL) {
        return NULL;
    }
    sink out = {buf, measure.len + 1, 0};
    render_into(t, &out);
    return buf;
}

void tally_string_free(char *s) {
    free(s);
}
