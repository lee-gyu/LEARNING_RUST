// 에디션별 결과: 2021에서 실패, 2024에서 통과
// 기대하는 에러(2021): E0700 — hidden type for `impl Iterator<Item = i32>` captures lifetime that does not appear in bounds
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2021 compile_fail/e0700_rpit_hidden_lifetime_2021.rs --out-dir /tmp/rust-p2-ch4   # 실패
//   rustc --edition 2024 compile_fail/e0700_rpit_hidden_lifetime_2021.rs --out-dir /tmp/rust-p2-ch4   # 통과
//
// `evens`가 돌려주는 반복자(숨은 타입)는 인자 `numbers`를 빌립니다. 2021 에디션에서 반환 위치의
// `impl Trait`(RPIT)는 바운드에 이름이 나오는 라이프타임만 캡처하는데, 이 시그니처의 바운드에는
// 라이프타임이 없습니다. 그래서 "숨은 타입이 바운드에 없는 라이프타임을 캡처한다"는 E0700입니다.
// 2024 에디션은 인자의 라이프타임까지 자동으로 캡처하므로 그대로 통과합니다.
// 해결(2021에 머문다면): 캡처할 라이프타임을 적습니다. `-> impl Iterator<Item = i32> + use<'_>`
// (정밀 캡처, 두 에디션에서 같은 뜻) 또는 예전 방식인 `+ '_`.

fn evens(numbers: &[i32]) -> impl Iterator<Item = i32> {
    numbers.iter().copied().filter(|n| n % 2 == 0) // → 2021에서 E0700
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    println!("{:?}", evens(&numbers).collect::<Vec<_>>());
}
