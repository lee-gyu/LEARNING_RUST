// 기대하는 에러: E0597
// 핵심 메시지: `source` does not live long enough
//
// TrackView<'a>는 자막 원문(source)을 빌린 뷰이고, library: Vec<TrackView<'a>>도 그 라이프타임을
// 물려받습니다. 원문은 안쪽 블록의 지역 값이라 블록 끝에서 drop되는데, library는 블록 밖에서도
// 쓰이므로 "뷰를 담은 컬렉션이 원문보다 오래 산다"가 되어 거부됩니다.
// 원문을 library보다 오래 사는 곳에 따로 보관하거나(examples/07_view_or_owned.rs의 2번 구역),
// 오래 보관할 것은 소유한 타입(Track, Cue)으로 바꿔 담습니다(같은 예제의 3·4번 구역).
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0597_view_outlives_source.rs --out-dir /tmp/er-ch14

fn read_subtitle(name: &str) -> String {
    format!("# {name}\n00:01 여기가 어디지?\n00:04 늦었어, 뛰어!")
}

struct CueView<'a> {
    time: &'a str,
    line: &'a str,
}

struct TrackView<'a> {
    title: &'a str,
    cues: Vec<CueView<'a>>,
}

fn parse_view(source: &str) -> TrackView<'_> {
    let mut lines = source.lines();
    let title = lines.next().unwrap_or("").trim_start_matches("# ");
    let cues = lines
        .filter_map(|l| l.split_once(' '))
        .map(|(time, line)| CueView { time, line })
        .collect();
    TrackView { title, cues }
}

fn main() {
    let mut library: Vec<TrackView<'_>> = Vec::new();
    {
        let source = read_subtitle("1화");
        library.push(parse_view(&source)); // E0597
    } // source가 여기서 drop됩니다.
    for track in &library {
        let opening = &track.cues[0];
        println!("{}: {} {}", track.title, opening.time, opening.line);
    }
}
