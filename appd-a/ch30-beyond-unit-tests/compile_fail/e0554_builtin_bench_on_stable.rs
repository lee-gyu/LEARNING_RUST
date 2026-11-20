// 기대하는 에러(stable 툴체인): E0554 `#![feature]` may not be used on the stable release channel
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --test --out-dir <임시 디렉터리> compile_fail/e0554_builtin_bench_on_stable.rs
//
// 표준 라이브러리에 딸린 벤치마크 하네스(`#[bench]`, `test::Bencher`)는 불안정 기능 `test`에 속해서
// nightly에서만 쓸 수 있습니다. stable에서는 첫 줄의 `#![feature(test)]`에서 바로 멈춥니다.
// 이 줄을 지우면 이번에는 `extern crate test`, `use test::Bencher`, `#[bench]`, `Bencher`, `b.iter`,
// `test::black_box` 여섯 곳에서 E0658 use of unstable library feature `test`가 납니다.
//
// stable에서 벤치마크를 쓰려면 `criterion` 같은 크레이트를 dev-dependency로 두고
// `benches/`의 타깃에 `harness = false`를 줍니다(projects/tinytlv/benches/codec.rs).

#![feature(test)]
extern crate test;

pub fn checksum(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |acc, &b| acc.rotate_left(5) ^ u32::from(b))
}

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_checksum(b: &mut Bencher) {
        let data = vec![7u8; 4096];
        b.iter(|| checksum(test::black_box(&data)));
    }
}
