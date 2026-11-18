// 이 예제가 보여 주는 것: Iterator 위에 덧붙는 보조 트레이트 — DoubleEndedIterator(next_back, rev, rfind, rposition),
//                        ExactSizeIterator(len)와 size_hint, 어댑터를 거친 뒤에도 그 성질이 남는지,
//                        컬렉션 메서드가 돌려주는 반복자 구조체(Drain, btree_map::Range, btree_set::Intersection),
//                        직접 만든 반복자에 두 트레이트 구현하기
// 실행: cargo run -p p1-ch5-iterators --example 07_double_ended
//
// rev()의 시그니처는 fn rev(self) -> Rev<Self> where Self: DoubleEndedIterator 입니다.
// "뒤에서도 꺼낼 수 있는 반복자"에만 쓸 수 있다는 조건이 붙어 있으므로, rev()가 되는지는 반복자의 타입이 정합니다.
// type_name 출력은 툴체인에 따라 바뀔 수 있습니다.

use std::any::type_name;
use std::collections::{BTreeMap, BTreeSet};

fn type_of<T>(_: &T) -> &'static str {
    type_name::<T>()
}

// 1쪽부터 last쪽까지 쪽 번호를 내놓는 반복자입니다. 아직 내놓지 않은 쪽은 front..=back이고, front > back이면 끝입니다.
struct Pages {
    front: u32,
    back: u32,
}

impl Pages {
    fn new(last: u32) -> Pages {
        Pages {
            front: 1,
            back: last,
        }
    }
}

impl Iterator for Pages {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.front > self.back {
            return None;
        }
        self.front += 1;
        Some(self.front - 1)
    }

    // 기본 구현은 (0, None)("모름")입니다. ExactSizeIterator를 구현하려면 정확한 값을 돌려주도록 재정의해야 합니다.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = if self.front > self.back {
            0
        } else {
            (self.back - self.front + 1) as usize
        };
        (n, Some(n))
    }
}

impl DoubleEndedIterator for Pages {
    // 앞의 next와 같은 상태(front..=back)를 뒤쪽에서 줄입니다. 양쪽이 만나면 둘 다 None을 돌려줍니다.
    fn next_back(&mut self) -> Option<u32> {
        if self.front > self.back {
            return None;
        }
        self.back -= 1;
        Some(self.back + 1)
    }
}

// 필수 메서드가 없습니다. len()은 size_hint를 써서 기본 구현으로 따라옵니다.
impl ExactSizeIterator for Pages {}

fn main() {
    println!("== 1. next_back: 뒤에서 꺼내기 ==");
    let mut it = [1, 2, 3, 4, 5].iter();
    let order = [it.next(), it.next_back(), it.next(), it.next_back()];
    let middle: Vec<&i32> = it.collect();
    println!("앞, 뒤, 앞, 뒤: {order:?}, 남은 것: {middle:?}");
    // 출력: 앞, 뒤, 앞, 뒤: [Some(1), Some(5), Some(2), Some(4)], 남은 것: [3]
    // 앞과 뒤는 같은 원소들을 양쪽에서 줄여 갑니다. 둘이 만나면 끝입니다.

    for word in ["기러기", "토마토", "반복자"] {
        let palindrome = word.chars().eq(word.chars().rev());
        println!("{word}: 회문인가? {palindrome}");
    }
    // 출력: 기러기: 회문인가? true
    // 출력: 토마토: 회문인가? true
    // 출력: 반복자: 회문인가? false

    let log = [
        "INFO 시작",
        "WARN 느림",
        "INFO 처리",
        "WARN 재시도",
        "INFO 종료",
    ];
    println!(
        "rfind(마지막 WARN): {:?}, rposition: {:?}",
        log.iter().rfind(|l| l.starts_with("WARN")),
        log.iter().rposition(|l| l.starts_with("WARN"))
    );
    // 출력: rfind(마지막 WARN): Some("WARN 재시도"), rposition: Some(3)
    // rposition은 뒤에서 찾지만 돌려주는 순번은 앞에서부터 센 값입니다.

    println!("\n== 2. 어댑터를 거친 뒤의 rev ==");
    let letters = ['a', 'b', 'c', 'd'];
    // map과 filter는 안쪽이 DoubleEndedIterator면 자신도 DoubleEndedIterator입니다.
    let upper_rev: String = letters
        .iter()
        .filter(|c| **c != 'b')
        .map(|c| c.to_ascii_uppercase())
        .rev()
        .collect();
    println!("filter + map + rev: {upper_rev}");
    // 출력: filter + map + rev: DCA

    // enumerate().rev()와 rev().enumerate()는 다릅니다. 앞의 것은 원래 순번을 유지하고, 뒤의 것은 뒤집은 뒤 0부터 셉니다.
    let a: Vec<(usize, &char)> = letters.iter().enumerate().rev().collect();
    let b: Vec<(usize, &char)> = letters.iter().rev().enumerate().collect();
    println!("enumerate().rev(): {a:?}");
    println!("rev().enumerate(): {b:?}");
    // 출력: enumerate().rev(): [(3, 'd'), (2, 'c'), (1, 'b'), (0, 'a')]
    // 출력: rev().enumerate(): [(0, 'd'), (1, 'c'), (2, 'b'), (3, 'a')]
    // Enumerate가 뒤에서 꺼내려면 "맨 뒤 원소의 순번"을 알아야 하므로 안쪽이 ExactSizeIterator여야 합니다.
    // 슬라이스 반복자는 길이를 알지만 Chars는 모르므로 "한글".chars().enumerate().rev()는 E0277입니다
    // (compile_fail/e0277_rev_needs_exact_size.rs). 글자의 바이트 위치가 필요하면 char_indices().rev()를 씁니다.
    let from_back: Vec<(usize, char)> = "한글".char_indices().rev().collect();
    println!("char_indices().rev(): {from_back:?}");
    // 출력: char_indices().rev(): [(3, '글'), (0, '한')]

    println!("\n== 3. ExactSizeIterator와 size_hint ==");
    let v = vec![10, 20, 30, 40, 50];
    let mut it = v.iter();
    it.next();
    println!(
        "len: 처음 {}, 하나 꺼낸 뒤 {}, map 뒤 {}, skip(2) 뒤 {}",
        v.iter().len(),
        it.len(),
        v.iter().map(|x| x * 2).len(),
        v.iter().skip(2).len()
    );
    // 출력: len: 처음 5, 하나 꺼낸 뒤 4, map 뒤 5, skip(2) 뒤 3
    // len()은 "남은" 원소 수입니다. filter 뒤에는 몇 개가 남을지 모르므로 len()이 없습니다(대신 count()로 세어 봅니다).

    // size_hint는 (최소 개수, Some(최대 개수) 또는 모르면 None)입니다. 모든 반복자가 가지고 있습니다.
    println!("size_hint:");
    println!("  v.iter()            {:?}", v.iter().size_hint());
    println!(
        "  filter              {:?}",
        v.iter().filter(|x| **x > 25).size_hint()
    );
    println!("  chain(자기 자신)    {:?}", v.iter().chain(&v).size_hint());
    println!("  (1..)               {:?}", (1..).size_hint());
    println!("  (1..).take(3)       {:?}", (1..).take(3).size_hint());
    println!(
        "  \"가나다abc\".chars() {:?}",
        "가나다abc".chars().size_hint()
    );
    // 출력: size_hint:
    // 출력:   v.iter()            (5, Some(5))
    // 출력:   filter              (0, Some(5))
    // 출력:   chain(자기 자신)    (10, Some(10))
    // 출력:   (1..)               (18446744073709551615, None)
    // 출력:   (1..).take(3)       (3, Some(3))
    // 출력:   "가나다abc".chars() (3, Some(12))
    // chars()는 12바이트 문자열에 글자가 몇 개인지 모르지만, 한 글자가 1~4바이트이므로 3~12개라는 것은 압니다.
    // collect는 이 최소값을 보고 Vec의 용량을 미리 확보할 수 있습니다.

    println!("\n== 4. 컬렉션이 돌려주는 반복자 구조체 ==");
    let mut queue = vec![1, 2, 3, 4, 5, 6];
    let drained = queue.drain(1..4);
    println!("drain 타입: {}", type_of(&drained));
    // 출력: drain 타입: alloc::vec::drain::Drain<'_, i32>
    // 문서의 Drain 페이지에는 Iterator, DoubleEndedIterator, ExactSizeIterator 구현이 모두 있으므로 len과 rev를 쓸 수 있습니다.
    println!("drain 길이: {}", drained.len());
    let taken: Vec<i32> = drained.rev().collect();
    println!("뒤에서부터 꺼냄: {taken:?}, 남은 queue: {queue:?}");
    // 출력: drain 길이: 3
    // 출력: 뒤에서부터 꺼냄: [4, 3, 2], 남은 queue: [1, 5, 6]

    let ledger: BTreeMap<&str, i32> =
        BTreeMap::from([("11-03", 4500), ("11-10", 8000), ("11-21", 5500)]);
    let range = ledger.range("11-01".."11-15");
    println!("range 타입: {}", type_of(&range));
    // 출력: range 타입: alloc::collections::btree::map::Range<'_, &str, i32>
    // btree_map::Range는 DoubleEndedIterator지만 ExactSizeIterator는 아닙니다. 범위 안에 몇 개가 있는지 세어 보기 전에는 모릅니다.
    println!(
        "범위의 마지막 항목: {:?}",
        ledger.range("11-01".."11-15").next_back()
    );
    // 출력: 범위의 마지막 항목: Some(("11-10", 8000))

    let a: BTreeSet<i32> = BTreeSet::from([1, 3, 5, 7]);
    let b: BTreeSet<i32> = BTreeSet::from([3, 4, 5, 6, 7]);
    let common = a.intersection(&b);
    println!("intersection 타입: {}", type_of(&common));
    // 출력: intersection 타입: alloc::collections::btree::set::Intersection<'_, i32>
    // Intersection은 Iterator만 구현하므로 rev()를 쓸 수 없습니다. 거꾸로 보려면 모은 뒤 뒤집습니다.
    let mut desc: Vec<&i32> = common.collect();
    desc.reverse();
    println!("교집합 내림차순: {desc:?}");
    // 출력: 교집합 내림차순: [7, 5, 3]

    println!("\n== 5. 직접 만든 반복자에 구현하기 ==");
    println!("앞에서:   {:?}", Pages::new(5).collect::<Vec<_>>());
    println!("rev:      {:?}", Pages::new(5).rev().collect::<Vec<_>>());
    println!("len:      {}", Pages::new(5).len());
    println!(
        "enumerate().rev(): {:?}",
        Pages::new(3).enumerate().rev().collect::<Vec<_>>()
    );
    // 출력: 앞에서:   [1, 2, 3, 4, 5]
    // 출력: rev:      [5, 4, 3, 2, 1]
    // 출력: len:      5
    // 출력: enumerate().rev(): [(2, 3), (1, 2), (0, 1)]
    // next_back과 size_hint를 채웠더니 rev, len, enumerate().rev()가 모두 따라왔습니다.
}
