#!/bin/sh
# pollcount의 검사 스크립트입니다. 로컬과 CI가 이 파일 하나로 같은 명령을 돌립니다(ch32 README 2.4절).
# POSIX sh 문법만 씁니다(macOS의 sh인 bash 3.2, Ubuntu의 sh인 dash에서 확인).
#
# 사용법(이 디렉터리에서):
#   sh ci.sh                  모든 단계를 차례로 돌리고 첫 실패에서 멈춥니다
#   sh ci.sh --keep-going     실패해도 끝까지 돌리고 마지막에 실패한 단계를 모아 보여 줍니다
#   sh ci.sh fmt clippy       고른 단계만 돌립니다
#   CI=true sh ci.sh          CI 모드: 설치되지 않은 도구·툴체인을 건너뛰지 않고 실패로 다룹니다
#                             (GitHub Actions는 CI=true를 늘 넣어 줍니다)
#
# 종료 코드: 모든 단계가 통과(로컬에서는 건너뜀 포함)하면 0, 하나라도 실패하면 1,
#            사용법이 틀리면 2입니다. 실패한 단계의 종료 코드는 단계마다 출력합니다.

set -u
cd "$(dirname "$0")" || exit 2

# MSRV 검사에 쓸 툴체인입니다. Cargo.toml의 rust-version과 맞아야 합니다(msrv 단계가 확인).
MSRV_TOOLCHAIN=1.98.1

# 외부 도구의 버전입니다. 워크플로(ci/github-workflow.yml)의 설치 줄과 맞춥니다.
DENY_VERSION=0.22.0
HACK_VERSION=0.6.46

# 원칙 "경고는 실패": 이 스크립트가 부르는 모든 cargo 명령(clippy, test, doc 포함)에서
# 경고가 있으면 실패합니다. Cargo 1.97부터 동작하고, RUSTFLAGS와 달리 다시 컴파일하지 않습니다.
CARGO_BUILD_WARNINGS=deny
export CARGO_BUILD_WARNINGS

ALL_STEPS="lockfile fmt conventions clippy test examples docs features msrv deny advisories hack"

KEEP_GOING=0
STEPS=""
for arg in "$@"; do
  case $arg in
    --keep-going) KEEP_GOING=1 ;;
    -*) echo "알 수 없는 옵션: $arg" >&2; exit 2 ;;
    *)
      case " $ALL_STEPS " in
        *" $arg "*) STEPS="$STEPS $arg" ;;
        *) echo "알 수 없는 단계: $arg (단계: $ALL_STEPS)" >&2; exit 2 ;;
      esac
      ;;
  esac
done
[ -n "$STEPS" ] || STEPS=$ALL_STEPS

in_ci() { [ "${CI:-}" = "true" ]; }

# 명령을 보여 주고 실행합니다. 로그만 보고도 같은 명령을 손으로 다시 돌릴 수 있게 합니다.
run() {
  echo "    \$ $*"
  "$@"
}

# 도구가 없을 때: 로컬에서는 건너뛰고(종료 코드 0), CI에서는 실패로 다룹니다(1).
# CI에서 건너뛰면 아무것도 검사하지 않은 초록 빌드가 됩니다.
missing() {
  if in_ci; then
    echo "    $1 없음: CI에서는 건너뛸 수 없습니다 (준비: $2)"
    return 1
  fi
  echo "    $1 없음: 건너뜀 (설치: $2)"
  SKIPPED="$SKIPPED $CURRENT"
  return 0
}

# --- 단계 -------------------------------------------------------------------
# 단계마다 함수 하나입니다. 모든 cargo 명령에 --locked를 붙여, 어느 단계도
# Cargo.lock을 몰래 고치지 못하게 합니다.

# Cargo.lock이 Cargo.toml과 맞는지 가장 먼저 확인합니다. 컴파일하지 않고 의존성만 해석합니다.
step_lockfile() {
  echo "    \$ cargo metadata --locked --format-version 1 > /dev/null"
  cargo metadata --locked --format-version 1 > /dev/null
}

step_fmt() {
  run cargo fmt --check
}

# 프로젝트만의 관례: Cargo.toml의 version에 해당하는 절이 CHANGELOG.md에 있어야 합니다.
step_conventions() {
  version=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)
  if grep -q -F -x "## $version" CHANGELOG.md; then
    echo "    CHANGELOG.md에 '## $version' 절이 있습니다"
  else
    echo "    CHANGELOG.md에 '## $version' 절이 없습니다. 이 버전의 변경 내용을 적어 주세요"
    return 1
  fi
}

step_clippy() {
  run cargo clippy --locked --all-targets --all-features
}

# --all-targets는 문서 테스트를 포함하지 않으므로 --doc을 따로 돌립니다.
step_test() {
  run cargo test --locked --all-targets --all-features &&
    run cargo test --locked --doc --all-features
}

# cargo test는 예제 프로그램을 빌드만 하므로 하나씩 실행합니다.
step_examples() {
  for file in examples/*.rs; do
    name=$(basename "$file" .rs)
    run cargo run --locked --quiet --all-features --example "$name" || return
  done
}

# docs.rs와 같은 기능(Cargo.toml의 [package.metadata.docs.rs])으로 문서를 만듭니다.
# 깨진 문서 링크 같은 rustdoc 경고도 CARGO_BUILD_WARNINGS=deny로 실패가 됩니다.
step_docs() {
  run cargo doc --locked --no-deps --all-features
}

# 기능 조합: 기능 없음과 기능 하나씩(전부 켠 조합은 clippy 단계가 맡습니다).
# 기능 이름은 Cargo.toml의 [features] 표에서 읽으므로 목록을 따로 관리하지 않습니다.
feature_names() {
  sed -n '/^\[features\]/,/^\[/p' Cargo.toml |
    sed -n 's/^\([A-Za-z0-9_-]*\) *=.*/\1/p' |
    grep -v -x default
}

step_features() {
  run cargo clippy --locked --all-targets --no-default-features || return
  for feature in $(feature_names); do
    run cargo clippy --locked --all-targets --no-default-features --features "$feature" || return
  done
}

step_msrv() {
  declared=$(sed -n 's/^rust-version = "\(.*\)"$/\1/p' Cargo.toml)
  case $MSRV_TOOLCHAIN in
    "$declared" | "$declared".*) ;;
    *)
      echo "    Cargo.toml의 rust-version($declared)과 MSRV_TOOLCHAIN($MSRV_TOOLCHAIN)이 다릅니다"
      return 1
      ;;
  esac
  command -v rustup > /dev/null 2>&1 ||
    { missing rustup "https://rustup.rs"; return; }
  # rustup은 설치되지 않은 툴체인을 이름으로 가리키기만 해도 내려받으므로, 목록으로만 확인합니다.
  rustup toolchain list | grep -q "^$MSRV_TOOLCHAIN-" ||
    { missing "툴체인 $MSRV_TOOLCHAIN" "rustup toolchain install $MSRV_TOOLCHAIN --profile minimal"; return; }
  # 경고 목록은 컴파일러 버전마다 다르므로, 경고는 고정한 툴체인 하나에서만 판정합니다.
  (
    CARGO_BUILD_WARNINGS=warn
    export CARGO_BUILD_WARNINGS
    run cargo "+$MSRV_TOOLCHAIN" check --locked --all-targets --all-features
  )
}

# 아래 셋은 이 저장소에 설치되어 있지 않아 실행해 확인하지 못한 단계입니다.
step_deny() {
  command -v cargo-deny > /dev/null 2>&1 ||
    { missing cargo-deny "cargo install --locked cargo-deny@$DENY_VERSION"; return; }
  run cargo deny check bans licenses sources
}

# 보안 권고는 코드가 그대로여도 새로 붙으므로 CI에서는 정기 예약 작업에서 돌립니다.
step_advisories() {
  command -v cargo-deny > /dev/null 2>&1 ||
    { missing cargo-deny "cargo install --locked cargo-deny@$DENY_VERSION"; return; }
  run cargo deny check advisories
}

step_hack() {
  command -v cargo-hack > /dev/null 2>&1 ||
    { missing cargo-hack "cargo install --locked cargo-hack@$HACK_VERSION"; return; }
  run cargo hack check --feature-powerset --all-targets
}

# --- 실행 -------------------------------------------------------------------

if in_ci; then mode="CI"; else mode="로컬"; fi
echo "모드: $mode, $(rustc --version), $(cargo --version)"

FAILED=""
SKIPPED=""
NOT_RUN=""
stopped=0
for step in $STEPS; do
  if [ "$stopped" -eq 1 ]; then
    NOT_RUN="$NOT_RUN $step"
    continue
  fi
  CURRENT=$step
  echo "==> $step"
  "step_$step"
  code=$?
  if [ "$code" -ne 0 ]; then
    echo "[실패] $step (종료 코드 $code)"
    FAILED="$FAILED $step($code)"
    [ "$KEEP_GOING" -eq 1 ] || stopped=1
  fi
done

echo "==> 요약"
[ -z "$SKIPPED" ] || echo "  건너뜀:$SKIPPED"
[ -z "$NOT_RUN" ] || echo "  실행하지 않음:$NOT_RUN"
if [ -n "$FAILED" ]; then
  echo "  실패:$FAILED"
  exit 1
fi
echo "  모든 단계 통과"
