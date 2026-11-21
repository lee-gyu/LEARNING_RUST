// 부록 B ch13 · Q13.6 — 05_odr_main.cpp와 함께 링크합니다. 명령과 설명은 05_odr_main.cpp에 있습니다.

inline int limit() { return 20; } // 같은 이름, 다른 정의: ODR 위반

int other_limit() { return limit(); }
