//! `criterion` 벤치마크: 레코드 수에 따라 `encode`와 `parse`가 얼마나 걸리는지 잽니다.
//!
//! 실행: cargo bench
//! 한 번씩만 실행해 깨지지 않았는지 보기: cargo test --benches
//!
//! `Cargo.toml`의 `[[bench]] harness = false` 덕분에 내장 벤치 하네스 대신
//! `criterion_main!`이 만든 `main`이 실행됩니다.

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tinytlv::{Record, encode, parse};

/// 값 16바이트짜리 레코드 `n`개
fn records(n: usize) -> Vec<Record> {
    (0..n)
        .map(|i| Record::new((i % 256) as u8, [(i % 251) as u8; 16]).unwrap())
        .collect()
}

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");
    for n in [10, 1000] {
        let input = records(n);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &input, |b, input| {
            // 반복마다 입력을 black_box로 감싸, 컴파일러가 입력을 안다고 보고 계산을 미리 해 두거나
            // 반복문 밖으로 빼내지 못하게 합니다. 클로저가 돌려준 값은 criterion이 black_box로 받아
            // "쓰이지 않는 결과"로 지워지지 않게 합니다(criterion 0.8.2 소스에서 확인).
            b.iter(|| encode(black_box(input)))
        });
    }
    group.finish();
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    for n in [10, 1000] {
        let bytes = encode(&records(n));
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &bytes, |b, bytes| {
            b.iter(|| parse(black_box(bytes)))
        });
    }
    group.finish();
}

/// 이 실습에서는 검증 시간을 줄이려고 기본값(예열 3초, 측정 5초, 표본 100개)보다 짧게 잽니다.
fn quick() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1))
        .sample_size(20)
}

criterion_group! {
    name = benches;
    config = quick();
    targets = bench_encode, bench_parse
}
criterion_main!(benches);
