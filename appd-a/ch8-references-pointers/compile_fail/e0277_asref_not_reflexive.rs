// 기대 에러: E0277
//   the trait bound `Track: AsRef<Track>` is not satisfied
//
// `AsRef`에는 `impl<T> AsRef<T> for T` 같은 반사 구현이 없습니다. 그래서 `T: AsRef<Track>`으로
// 받는 함수에는 `Track` 자신의 목록도, `&Track`의 목록도 넘길 수 없습니다(`&Track`은
// `Track: AsRef<Track>`일 때만 `AsRef<Track>`입니다). 같은 타입의 소유한 형태와 빌린 형태를
// 모두 받으려면 `Borrow<Track>`으로 받습니다(예제 03의 `total_secs`).
//
// 확인: rustc --edition 2024 compile_fail/e0277_asref_not_reflexive.rs --out-dir /tmp/er-ch8

struct Track {
    title: &'static str,
    secs: u32,
}

fn total_secs_as_ref<T: AsRef<Track>>(list: &[T]) -> u32 {
    list.iter().map(|t| t.as_ref().secs).sum()
}

fn main() {
    let library = vec![
        Track { title: "새벽 산책", secs: 214 },
        Track { title: "비 오는 오후", secs: 177 },
    ];
    let playlist: Vec<&Track> = library.iter().collect();
    println!("{}", playlist[0].title);
    println!("{}", total_secs_as_ref(&library)); // E0277: Track: AsRef<Track>
    println!("{}", total_secs_as_ref(&playlist)); // E0277: &Track: AsRef<Track>
}
