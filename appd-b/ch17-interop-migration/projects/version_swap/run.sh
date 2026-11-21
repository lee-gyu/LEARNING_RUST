#!/bin/sh
# C 프로그램(csrc/main.c)의 semver_parse를 C 구현과 Rust 구현으로 각각 링크하고, 같은 입력에서 출력을 비교합니다.
# 필요한 것: C 컴파일러(cc)와 cargo. 외부 크레이트를 쓰지 않으므로 네트워크가 필요 없습니다.
set -eu
cd "$(dirname "$0")"
mkdir -p target

echo "== 1. 원래 프로그램: main.c + semver.c"
cc -std=c11 -Wall -Wextra -o target/app_c csrc/main.c csrc/semver.c
./target/app_c < inputs.txt | tee target/out_c.txt

echo "== 2. 바꿔 끼운 프로그램: main.c + Rust 정적 라이브러리"
cargo build --release -q
cc -std=c11 -Wall -Wextra -o target/app_rust csrc/main.c target/release/libsemver_rs.a
./target/app_rust < inputs.txt > target/out_rust.txt

echo "== 3. 두 출력의 차이(diff)"
if diff target/out_c.txt target/out_rust.txt; then
    echo "차이 없음"
fi
