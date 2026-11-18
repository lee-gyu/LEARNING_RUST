// 이 예제가 보여 주는 것: API 문서의 메서드 시그니처(signature)를 읽고 그대로 호출하는 법
//   - `self` / `&self` / `&mut self`가 호출하는 쪽에 요구하는 것
//   - 제네릭과 `where` 절, `impl Trait` 매개변수
//   - 반환 타입이 `Option`/`Result`인 메서드
//   - `const fn`, 그리고 `Vec`/`String`에서 쓰는 "Methods from Deref"
// 실행: cargo run -p p1-ch1-std-overview --example 03_reading_signatures

// `const fn`인 메서드는 상수의 값을 계산하는 데 쓸 수 있습니다.
// 문서: `pub const fn pow(self, exp: u32) -> u32` (1.0.0, const: 1.50.0)
const KIB: u32 = 2u32.pow(10);

fn main() {
    // ── 1. self / &self / &mut self ──────────────────────────────────────
    // 문서: `pub const fn to_ascii_uppercase(&self) -> char`
    // `&self`는 값을 빌려서 읽기만 합니다. 결과는 새 `char`로 돌려주고 원래 값은 그대로입니다.
    let c = 'r';
    let upper = c.to_ascii_uppercase();
    println!("c = {c}, upper = {upper}"); // 출력: c = r, upper = R

    // 문서: `pub const fn make_ascii_uppercase(&mut self)`
    // `&mut self`는 그 자리에서 값을 바꿉니다. 그래서 변수가 `mut`여야 하고, 반환값이 없습니다.
    let mut d = 'r';
    d.make_ascii_uppercase();
    println!("d = {d}"); // 출력: d = R

    // 문서: `pub const fn abs(self) -> i32`
    // `self`는 값을 통째로 받습니다. `i32`는 `Copy`라서 복사본이 넘어가므로 원래 변수도 계속 쓸 수 있습니다.
    let n = -5_i32;
    println!("n.abs() = {}, n = {n}", n.abs()); // 출력: n.abs() = 5, n = -5

    // 문서: `pub fn map<F, U>(self, f: F) -> [U; N] where F: FnMut(T) -> U` (배열의 메서드)
    // 원소가 `Copy`가 아닌 배열에 `self` 메서드를 부르면 배열의 소유권이 메서드로 이동합니다.
    let names = [String::from("ferris"), String::from("corro")];
    let lengths = names.map(|s| s.len());
    println!("lengths = {lengths:?}"); // 출력: lengths = [6, 5]
    // println!("{names:?}"); // E0382: `names`는 `map`으로 이동했습니다.

    // 메서드는 첫 매개변수가 `self`인 연관 함수이므로 `타입::메서드(값, 인자)`로도 부를 수 있습니다.
    // 시그니처를 그대로 따라 쓰는 모양이라, 문서를 읽을 때 무엇이 `self`인지 확인하기 좋습니다.
    println!("{} {}", 2_i32.pow(10), i32::pow(2, 10)); // 출력: 1024 1024

    // ── 2. 제네릭과 where 절 ─────────────────────────────────────────────
    let nums = [10, 20, 30, 40];

    // 문서: `pub fn contains(&self, x: &T) -> bool where T: PartialEq`
    // `x: &T`이므로 값이 아니라 참조를 넘깁니다. `where T: PartialEq`는 "원소끼리 ==로 비교할 수 있어야 한다"는 조건입니다.
    println!("contains(&30) = {}", nums.contains(&30)); // 출력: contains(&30) = true

    // 문서: `pub fn get<I>(&self, index: I) -> Option<&<I as SliceIndex<[T]>>::Output>
    //        where I: SliceIndex<[T]>`
    // `I`는 "슬라이스의 인덱스로 쓸 수 있는 타입"입니다. `usize`를 넘기면 원소 하나(`&T`),
    // 범위를 넘기면 부분 슬라이스(`&[T]`)가 `Output`이 됩니다. 반환 타입이 인자 타입에 따라 정해집니다.
    let one: Option<&i32> = nums.get(1);
    let part: Option<&[i32]> = nums.get(1..3);
    let out_of_range: Option<&[i32]> = nums.get(2..9);
    println!("{one:?} {part:?} {out_of_range:?}"); // 출력: Some(20) Some([20, 30]) None

    // 문서: `pub fn fill(&mut self, value: T) where T: Clone`
    // 같은 값을 모든 칸에 넣으려면 값을 여러 번 복제해야 하므로 `T: Clone`을 요구합니다.
    let mut grid = [0; 5];
    grid.fill(7);
    println!("grid = {grid:?}"); // 출력: grid = [7, 7, 7, 7, 7]

    // ── 3. impl Trait 매개변수 ───────────────────────────────────────────
    // 문서: `pub fn pop_if(&mut self, predicate: impl FnOnce(&mut T) -> bool) -> Option<T>` (Vec, 1.86.0)
    // `impl FnOnce(&mut T) -> bool`은 "`&mut T`를 받아 `bool`을 돌려주는 클로저면 무엇이든"이라는 뜻입니다.
    let mut stack = vec![1, 2, 3, 4];
    let popped = stack.pop_if(|top| *top % 2 == 0);
    println!("popped = {popped:?}, stack = {stack:?}"); // 출력: popped = Some(4), stack = [1, 2, 3]

    // 문서: `pub fn then<T, F>(self, f: F) -> Option<T> where F: FnOnce() -> T` (bool)
    // 같은 뜻을 이름 있는 타입 매개변수 `F`와 `where` 절로 적은 모양입니다.
    let msg = (stack.len() == 3).then(|| format!("원소 {}개", stack.len()));
    println!("msg = {msg:?}"); // 출력: msg = Some("원소 3개")

    // ── 4. Option과 Result를 돌려주는 메서드 ────────────────────────────────
    // `Option`: 실패할 수 있지만 이유를 따로 알려 줄 필요가 없을 때 (`None` 하나로 충분)
    // 문서: `pub const fn checked_div(self, rhs: i32) -> Option<i32>` — 0으로 나누거나 오버플로면 None
    match 10_i32.checked_div(0) {
        Some(q) => println!("몫 = {q}"),
        None => println!("checked_div(0) = None"), // 출력: checked_div(0) = None
    }

    // `Result`: 실패한 이유를 에러 값으로 알려 줄 때
    // 문서: `pub const fn from_str_radix(src: &str, radix: u32) -> Result<i32, ParseIntError>`
    match i32::from_str_radix("1fz", 16) {
        Ok(v) => println!("값 = {v}"),
        Err(e) => println!("에러: {e}"), // 출력: 에러: invalid digit found in string
    }

    // ── 5. Methods from Deref ───────────────────────────────────────────
    // `Vec<T>`는 `Deref<Target = [T]>`를 구현합니다. 그래서 `Vec` 문서의 "Methods from Deref<Target = [T]>"
    // 절에 슬라이스의 메서드가 모두 나오고, `Vec`에서 바로 부를 수 있습니다(역참조 강제 변환, Part 0 ch14).
    let v = vec![3, 1, 4, 1, 5];
    println!("first = {:?}, last = {:?}", v.first(), v.last()); // 출력: first = Some(3), last = Some(5)
    println!("v.contains(&4) = {}", v.contains(&4)); // 출력: v.contains(&4) = true
    // 슬라이스가 필요하다는 것을 드러내고 싶으면 `as_slice()`나 `&v[..]`로 직접 꺼냅니다.
    let s: &[i32] = v.as_slice();
    println!("s.len() = {}", s.len()); // 출력: s.len() = 5

    // `String`은 `Deref<Target = str>`를 구현하므로 `str`의 메서드를 부를 수 있습니다.
    // `to_uppercase`는 `String` 페이지의 "Methods from Deref<Target = str>" 절에 있습니다.
    // (문자열 메서드 자체는 `ch3-strings-fmt`에서 다룹니다.)
    let name = String::from("ferris");
    println!("{}", name.to_uppercase()); // 출력: FERRIS

    // ── 6. const fn ────────────────────────────────────────────────────
    println!("KIB = {KIB}"); // 출력: KIB = 1024
}
