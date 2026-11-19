//! 구조체에 참조를 담을지 소유할지: 잠깐 들여다보는 뷰에는 참조, 오래 보관하는 값에는 소유.
//!
//! 실행: cargo run -p er-ch14-lifetimes --example 07_view_or_owned
//!
//! 참조를 담은 구조체는 가리키는 값보다 오래 살 수 없고, 그 제약은 라이프타임 매개변수로
//! 그 구조체를 담는 모든 타입에 번집니다. 복사가 없다는 장점과 이 제약을 함께 따져 고릅니다.

/// 가짜 파일 시스템: 자막 파일을 읽어 새 String으로 돌려줍니다. 첫 줄은 제목입니다.
fn read_subtitle(name: &str) -> String {
    let body = match name {
        "1화" => "00:01 여기가 어디지?\n00:04 늦었어, 뛰어!\n00:09 문이 닫히기 전에 타야 해",
        "2화" => "00:02 다시 만났네\n00:06 우산 가져왔어?",
        _ => "",
    };
    format!("# {name}\n{body}")
}

// ---- 뷰: 원문을 빌려 가리킵니다 ----

/// 자막 한 줄. 시각과 대사는 원문의 일부를 가리킵니다.
#[derive(Clone, Copy)]
struct CueView<'a> {
    time: &'a str,
    line: &'a str,
}

/// 뷰를 담는 타입에도 라이프타임 매개변수가 번집니다.
/// 제목과 대사가 모두 같은 원문의 일부이므로 라이프타임은 하나입니다.
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

// ---- 소유: 필요한 글자를 복사해 가집니다 ----

struct Cue {
    time: String,
    line: String,
}

struct Track {
    title: String,
    cues: Vec<Cue>,
}

impl CueView<'_> {
    /// 오래 보관할 것만 골라 소유한 값으로 바꿉니다.
    fn to_owned_cue(self) -> Cue {
        Cue {
            time: self.time.to_string(),
            line: self.line.to_string(),
        }
    }
}

/// 소유한 판은 원문을 함수 안에서 읽고 버려도 됩니다. 뷰 판으로는 이런 함수를 만들 수 없습니다.
fn load(name: &str) -> Track {
    let source = read_subtitle(name); // 이 함수의 지역 값
    let view = parse_view(&source);
    Track {
        title: view.title.to_string(),
        cues: view.cues.into_iter().map(CueView::to_owned_cue).collect(),
    } // source는 여기서 drop되지만 Track은 아무것도 빌리지 않습니다.
}

fn main() {
    println!("1. 뷰: 원문이 살아 있는 동안 복사 없이 훑어봅니다");
    let source = read_subtitle("1화");
    let view = parse_view(&source);
    let wordiest = view.cues.iter().max_by_key(|c| c.line.chars().count());
    let wordiest = wordiest.map_or("", |c| c.line);
    println!(
        "    {}: 대사 {}줄, 가장 긴 대사 {wordiest:?}",
        view.title,
        view.cues.len()
    );
    println!("    대사를 위해 만든 String: 0개");
    // 출력:
    //     1화: 대사 3줄, 가장 긴 대사 "문이 닫히기 전에 타야 해"
    //     대사를 위해 만든 String: 0개

    println!("2. 뷰를 모아 두려면 원문을 더 오래 사는 곳에 따로 보관해야 합니다");
    // 원문 보관소(sources)는 뷰 목록(library)을 다 쓸 때까지 살아 있어야 합니다.
    let sources: Vec<String> = ["1화", "2화"].map(read_subtitle).to_vec();
    let library: Vec<TrackView<'_>> = sources.iter().map(|s| parse_view(s)).collect();
    for track in &library {
        println!("    {}: {}줄", track.title, track.cues.len());
    }
    // 출력:
    //     1화: 3줄
    //     2화: 2줄

    println!("3. 소유: 원문을 버려도 되므로 불러오는 함수가 단순해집니다");
    let owned: Vec<Track> = ["1화", "2화"].map(load).into_iter().collect();
    let strings: usize = owned.iter().map(|t| 1 + t.cues.len() * 2).sum();
    let opening = &owned[1].cues[0];
    println!(
        "    {}편 불러옴, Track에 든 String: {strings}개",
        owned.len()
    );
    println!(
        "    {} 첫 대사: {} {}",
        owned[1].title, opening.time, opening.line
    );
    // 출력:
    //     2편 불러옴, Track에 든 String: 12개
    //     2화 첫 대사: 00:02 다시 만났네

    println!("4. 섞어 쓰기: 뷰로 훑고, 오래 보관할 것만 소유로 바꿉니다");
    let mut bookmarks: Vec<Cue> = Vec::new();
    {
        let source = read_subtitle("2화"); // 이 블록이 끝나면 drop됩니다.
        let view = parse_view(&source);
        let questions = view.cues.iter().filter(|c| c.line.contains('?'));
        bookmarks.extend(questions.map(|c| c.to_owned_cue()));
    }
    for mark in &bookmarks {
        println!("    책갈피: {} {}", mark.time, mark.line);
    }
    // 출력:     책갈피: 00:06 우산 가져왔어?
}
