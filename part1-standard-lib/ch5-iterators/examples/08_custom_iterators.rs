// 이 예제가 보여 주는 것: 반복자를 "만드는 쪽"에 서기.
//                        std::iter의 함수(repeat, repeat_n, once, empty, from_fn, successors, zip, chain),
//                        impl Iterator를 돌려주는 함수, 직접 만든 컬렉션에 IntoIterator(값·&·&mut) 구현,
//                        FromIterator와 Extend 구현(그러면 collect, extend, Result로 collect까지 따라옵니다)
// 실행: cargo run -p p1-ch5-iterators --example 08_custom_iterators
//
// 반복자를 새로 만드는 방법은 세 가지입니다. (1) std::iter의 함수에 클로저나 값을 넘긴다,
// (2) 기존 반복자에 어댑터를 붙여 impl Iterator로 돌려준다, (3) 구조체를 만들어 트레이트를 직접 구현한다.
// 위에서 아래로 갈수록 손이 많이 가므로, 앞의 방법으로 되면 앞의 방법을 씁니다.

use std::collections::BTreeMap;
use std::iter;

#[derive(Debug)]
struct Song {
    title: String,
    secs: u32,
}

// Vec<Song>을 감싼 직접 만든 컬렉션입니다.
struct Playlist {
    songs: Vec<Song>,
}

impl Playlist {
    // 관례에 따라 iter()를 제공하고, 구체적인 반복자 타입을 그대로 돌려줍니다.
    // slice::Iter는 DoubleEndedIterator와 ExactSizeIterator이므로 호출하는 쪽에서 rev()와 len()을 쓸 수 있습니다.
    fn iter(&self) -> std::slice::Iter<'_, Song> {
        self.songs.iter()
    }

    fn total_secs(&self) -> u32 {
        self.iter().map(|s| s.secs).sum()
    }
}

// 1) for song in playlist — 소유권을 가져가 Song을 하나씩 내놓습니다. 안쪽 Vec의 IntoIter에 맡깁니다.
impl IntoIterator for Playlist {
    type Item = Song;
    type IntoIter = std::vec::IntoIter<Song>;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.into_iter()
    }
}

// 2) for song in &playlist — 빌려서 &Song을 내놓습니다. 'a는 "playlist를 빌린 동안"이라는 라이프타임입니다.
impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a Song;
    type IntoIter = std::slice::Iter<'a, Song>;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.iter()
    }
}

// 3) for song in &mut playlist — 가변으로 빌려서 &mut Song을 내놓습니다.
impl<'a> IntoIterator for &'a mut Playlist {
    type Item = &'a mut Song;
    type IntoIter = std::slice::IterMut<'a, Song>;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.iter_mut()
    }
}

// collect::<Playlist>()가 되게 합니다. 받은 것을 안쪽 Vec으로 모으기만 하면 됩니다.
impl FromIterator<Song> for Playlist {
    fn from_iter<I: IntoIterator<Item = Song>>(iter: I) -> Self {
        Playlist {
            songs: iter.into_iter().collect(),
        }
    }
}

// playlist.extend(...)가 되게 합니다.
impl Extend<Song> for Playlist {
    fn extend<I: IntoIterator<Item = Song>>(&mut self, iter: I) {
        self.songs.extend(iter);
    }
}

fn song(title: &str, secs: u32) -> Song {
    Song {
        title: title.to_string(),
        secs,
    }
}

// "제목,초" 한 줄을 Song으로 바꿉니다.
fn parse_song(line: &str) -> Result<Song, String> {
    let (title, secs) = line
        .split_once(',')
        .ok_or_else(|| format!("쉼표가 없음: {line:?}"))?;
    let secs = secs
        .trim()
        .parse()
        .map_err(|_| format!("초가 숫자가 아님: {line:?}"))?;
    Ok(song(title.trim(), secs))
}

// 어댑터를 붙인 반복자를 돌려줍니다. 클로저 타입에는 이름이 없으므로 impl Iterator로 적습니다.
fn multiples(n: u32) -> impl Iterator<Item = u32> {
    (1..).map(move |i| i * n) // 돌려준 뒤에도 n을 써야 하므로 move로 복사해 넣습니다.
}

// 입력을 빌리는 반복자도 돌려줄 수 있습니다. 2024 에디션에서는 반환 타입의 impl Trait가 입력의 라이프타임을
// 자동으로 붙잡으므로 따로 적지 않아도 됩니다(Part 2 ch4-rust-2024-language에서 다룹니다).
fn long_words(text: &str, min: usize) -> impl Iterator<Item = &str> {
    text.split_whitespace()
        .filter(move |w| w.chars().count() >= min)
}

// impl Iterator는 "Iterator라는 것만" 약속하므로, 실제 타입이 DoubleEndedIterator여도 호출하는 쪽은 rev()를 못 씁니다.
// 더 약속하고 싶으면 그 트레이트를 반환 타입에 적습니다.
fn long_words_both_ends(text: &str, min: usize) -> impl DoubleEndedIterator<Item = &str> {
    text.split_whitespace()
        .filter(move |w| w.chars().count() >= min)
}

fn main() {
    println!("== 1. std::iter의 함수 ==");
    // repeat(x): x를 끝없이 복제해 내놓습니다. zip의 짝으로 자주 씁니다.
    let board: BTreeMap<&str, u32> = ["가람", "나래"].into_iter().zip(iter::repeat(0)).collect();
    println!("repeat + zip:  {board:?}");
    // once(x): x 하나만 내놓습니다. repeat_n(x, n): x를 n번 내놓습니다. chain으로 이어 붙이기 좋습니다.
    let border: String = iter::once('+')
        .chain(iter::repeat_n('-', 8))
        .chain(iter::once('+'))
        .collect();
    println!("once + repeat_n: {border}");
    // empty(): 아무것도 내놓지 않습니다. 타입만 맞춘 빈 반복자가 필요할 때 씁니다.
    println!("empty: count = {}", iter::empty::<u32>().count());
    // 출력: repeat + zip:  {"가람": 0, "나래": 0}
    // 출력: once + repeat_n: +--------+
    // 출력: empty: count = 0

    // from_fn(클로저): 클로저가 곧 next입니다. 상태는 클로저가 캡처한 변수에 둡니다.
    let mut state = (0u64, 1u64);
    let fib: Vec<u64> = iter::from_fn(move || {
        let current = state.0;
        state = (state.1, state.0 + state.1);
        Some(current)
    })
    .take_while(|n| *n < 100)
    .collect();
    println!("from_fn(피보나치): {fib:?}");
    // 출력: from_fn(피보나치): [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]

    // successors(첫 값, |이전 값| 다음 값): 이전 값만으로 다음 값이 정해질 때 씁니다. None이면 끝납니다.
    let collatz: Vec<u32> = iter::successors(Some(6), |&n| match n {
        1 => None,
        n if n % 2 == 0 => Some(n / 2),
        n => Some(3 * n + 1),
    })
    .collect();
    let powers: Vec<u8> = iter::successors(Some(1u8), |n| n.checked_mul(2)).collect();
    println!("successors(콜라츠): {collatz:?}");
    println!("successors(u8 2의 거듭제곱): {powers:?}");
    // 출력: successors(콜라츠): [6, 3, 10, 5, 16, 8, 4, 2, 1]
    // 출력: successors(u8 2의 거듭제곱): [1, 2, 4, 8, 16, 32, 64, 128]
    // 256은 u8을 넘으므로 checked_mul이 None을 돌려주고, 거기서 끝납니다.

    // iter::zip, iter::chain: 메서드와 같지만 양쪽 모두 IntoIterator를 받습니다(chain 함수는 1.91부터 안정화).
    let names = ["가람", "나래"];
    let scores = [88, 72];
    let pairs: Vec<(&&str, &i32)> = iter::zip(&names, &scores).collect();
    let all: Vec<&str> = iter::chain(names, ["다온"]).collect();
    println!("iter::zip: {pairs:?}, iter::chain: {all:?}");
    // 출력: iter::zip: [("가람", 88), ("나래", 72)], iter::chain: ["가람", "나래", "다온"]

    println!("\n== 2. impl Iterator를 돌려주는 함수 ==");
    let first_five: Vec<u32> = multiples(7).take(5).collect();
    println!("multiples(7): {first_five:?}");
    let text = "반복자 어댑터는 게으르다 그래서 빠르다";
    let long: Vec<&str> = long_words(text, 4).collect();
    let long_rev: Vec<&str> = long_words_both_ends(text, 4).rev().collect();
    println!("long_words:            {long:?}");
    println!("long_words_both_ends:  {long_rev:?} (rev)");
    // 출력: multiples(7): [7, 14, 21, 28, 35]
    // 출력: long_words:            ["어댑터는", "게으르다"]
    // 출력: long_words_both_ends:  ["게으르다", "어댑터는"] (rev)
    // long_words(text, 4).rev()는 E0277입니다(`impl Iterator<Item = &str>: DoubleEndedIterator` is not satisfied).
    // 실제 타입(Filter<SplitWhitespace, _>)은 뒤에서 꺼낼 수 있지만, 반환 타입이 Iterator만 약속하기 때문입니다.
    let three: Vec<&str> = long_words_both_ends(text, 3).rev().collect();
    println!("세 글자 이상, 뒤에서부터: {three:?}");
    // 출력: 세 글자 이상, 뒤에서부터: ["빠르다", "그래서", "게으르다", "어댑터는", "반복자"]

    println!("\n== 3. 직접 만든 컬렉션의 IntoIterator 세 가지 ==");
    let mut playlist = Playlist {
        songs: vec![
            song("봄날", 245),
            song("밤편지", 253),
            song("Hype Boy", 179),
        ],
    };
    for s in &playlist {
        // &Playlist: IntoIterator → s는 &Song
        print!("{}({}초) ", s.title, s.secs);
    }
    println!("→ 합계 {}초", playlist.total_secs());
    for s in &mut playlist {
        // &mut Playlist: IntoIterator → s는 &mut Song. 라디오 편집본으로 10초씩 줄입니다.
        s.secs -= 10;
    }
    println!(
        "&mut로 고친 뒤 합계 {}초, 마지막 곡: {:?}",
        playlist.total_secs(),
        playlist.iter().next_back().map(|s| &s.title)
    );
    let titles: Vec<String> = playlist.into_iter().map(|s| s.title).collect(); // Playlist: IntoIterator → Song
    println!("값으로 꺼낸 제목: {titles:?}");
    // 출력: 봄날(245초) 밤편지(253초) Hype Boy(179초) → 합계 677초
    // 출력: &mut로 고친 뒤 합계 647초, 마지막 곡: Some("Hype Boy")
    // 출력: 값으로 꺼낸 제목: ["봄날", "밤편지", "Hype Boy"]

    println!("\n== 4. FromIterator와 Extend 구현 ==");
    let mut mix: Playlist = [("A", 200), ("B", 180)]
        .into_iter()
        .map(|(t, s)| song(t, s))
        .collect(); // FromIterator<Song> for Playlist
    mix.extend([song("C", 210)]); // Extend<Song> for Playlist
    println!(
        "collect + extend: {}곡, {}초",
        mix.songs.len(),
        mix.total_secs()
    );
    // 출력: collect + extend: 3곡, 590초

    // Result<V, E>: FromIterator<Result<T, E>> where V: FromIterator<T> 이므로,
    // Playlist에 FromIterator<Song>을 구현한 것만으로 Result<Playlist, String>으로도 모을 수 있습니다.
    let good: Result<Playlist, String> =
        ["가, 120", "나, 95"].into_iter().map(parse_song).collect();
    let bad: Result<Playlist, String> = ["가, 120", "나 95", "다, x"]
        .into_iter()
        .map(parse_song)
        .collect();
    match good {
        Ok(p) => println!("Ok: {:?}", p.songs),
        Err(e) => println!("Err: {e}"),
    }
    match bad {
        Ok(p) => println!("Ok: {:?}", p.songs),
        Err(e) => println!("Err: {e}"),
    }
    // 출력: Ok: [Song { title: "가", secs: 120 }, Song { title: "나", secs: 95 }]
    // 출력: Err: 쉼표가 없음: "나 95"
}
