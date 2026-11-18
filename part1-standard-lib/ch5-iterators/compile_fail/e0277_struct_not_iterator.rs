// 기대하는 에러: error[E0277]: `&Playlist` is not an iterator
// 핵심 메시지: help: the trait `Iterator` is not implemented for `&Playlist`
//             note: required for `&Playlist` to implement `IntoIterator`
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_struct_not_iterator.rs --out-dir /tmp/rust-p1-ch5
//
// for는 IntoIterator를 구현한 값만 받습니다. Playlist 안에 Vec이 들어 있어도 컴파일러는 그 사실로
// "Playlist를 어떻게 반복할지"를 추측하지 않습니다. 메시지가 "is not an iterator"인 이유는, 모든 Iterator가
// IntoIterator를 구현한다는 포괄 구현(impl<I: Iterator> IntoIterator for I)이 컴파일러가 시도해 본 유일한 길이기 때문입니다.
// 고치는 방법:
//   1) 안쪽 필드를 직접 반복합니다: for song in &playlist.songs { ... }
//   2) 컬렉션답게 만들려면 IntoIterator를 구현합니다(값·&·&mut 세 가지). examples/08_custom_iterators.rs의 Playlist를 보세요.
//      impl<'a> IntoIterator for &'a Playlist {
//          type Item = &'a String;
//          type IntoIter = std::slice::Iter<'a, String>;
//          fn into_iter(self) -> Self::IntoIter { self.songs.iter() }
//      }

struct Playlist {
    songs: Vec<String>,
}

fn main() {
    let playlist = Playlist {
        songs: vec![String::from("봄날"), String::from("밤편지")],
    };
    for song in &playlist {
        println!("{song}");
    }
}
