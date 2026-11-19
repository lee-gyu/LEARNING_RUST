//! 지침 5: 필드를 훑는 코드는 derive에 맡기고, 필드 이름 문자열로 값을 찾지 않는다
//!
//! 수강 기록(`Enrollment`)을 내보냅니다. 처음에는 필드가 셋이었고, 나중에 `certificate`를 더했습니다.
//! - 비권장: 필드 이름 문자열을 받아 값을 돌려주는 `field(name)`을 손으로 만들어 씁니다.
//! - 권장 1: 모든 필드를 보여 주는 일은 `#[derive(Debug)]`가 컴파일 시간에 만든 코드에 맡깁니다.
//! - 권장 2: 고른 열만 내보낼 때는 (머리글, 접근 함수) 표를 둡니다. 필드 이름은 컴파일러가 확인합니다.
//!
//! 실행: cargo run -p er-ch19-avoid-reflection --example 05_derive_over_field_lookup

/// derive(Debug)는 이 구조체의 필드 목록을 컴파일할 때 읽어 `Debug` 구현을 만듭니다.
/// 필드를 더하면 다음 컴파일에서 구현도 함께 바뀝니다. 실행 중에 필드를 찾는 일은 없습니다.
#[derive(Debug)]
struct Enrollment {
    learner: String,
    course: String,
    progress: u8,      // 진도율(%)
    certificate: bool, // 나중에 더한 필드: 수료증 발급 여부
}

impl Enrollment {
    /// 비권장: 다른 언어의 "필드 이름으로 값 읽기"를 흉내 낸 함수입니다.
    /// 이름이 틀려도, 새 필드를 빠뜨려도 컴파일러는 알 수 없고 `None`이 나올 뿐입니다.
    fn field(&self, name: &str) -> Option<String> {
        match name {
            "learner" => Some(self.learner.clone()),
            "course" => Some(self.course.clone()),
            "progress" => Some(self.progress.to_string()),
            _ => None, // certificate를 더할 때 이 함수는 고치지 않았습니다.
        }
    }
}

fn export_by_name(row: &Enrollment, columns: &[&str]) -> String {
    let cells: Vec<String> = columns
        .iter()
        .map(|name| row.field(name).unwrap_or_else(|| "?".to_string()))
        .collect();
    cells.join(",")
}

/// 권장 2: 열 하나 = (머리글, 값을 꺼내는 함수)
/// 접근 함수 안의 `e.progress`는 보통의 필드 접근이라, 이름이 틀리면 E0609로 컴파일되지 않습니다.
type Column = (&'static str, fn(&Enrollment) -> String);

const COLUMNS: [Column; 3] = [
    ("learner", |e| e.learner.clone()),
    ("progress", |e| e.progress.to_string()),
    ("certificate", |e| e.certificate.to_string()),
];

fn export_with_columns(row: &Enrollment, columns: &[Column]) -> String {
    let cells: Vec<String> = columns.iter().map(|(_, get)| get(row)).collect();
    cells.join(",")
}

fn main() {
    let rows = [
        Enrollment {
            learner: "김하나".to_string(),
            course: "러스트 입문".to_string(),
            progress: 80,
            certificate: false,
        },
        Enrollment {
            learner: "이두리".to_string(),
            course: "러스트 입문".to_string(),
            progress: 100,
            certificate: true,
        },
    ];

    println!("1. 비권장: 필드 이름 문자열로 꺼내기");
    // "progres"는 오타이고, "certificate"는 field()가 모르는 새 필드입니다.
    let wanted = ["learner", "progres", "certificate"];
    println!("    {}", wanted.join(","));
    for row in &rows {
        println!("    {}", export_by_name(row, &wanted));
    }
    // 출력:
    //     learner,progres,certificate
    //     김하나,?,?
    //     이두리,?,?

    println!("2. 권장: derive(Debug)가 만든 코드는 새 필드도 함께 보여 줌");
    for row in &rows {
        println!("    {row:?}");
    }
    // 출력:
    //     Enrollment { learner: "김하나", course: "러스트 입문", progress: 80, certificate: false }
    //     Enrollment { learner: "이두리", course: "러스트 입문", progress: 100, certificate: true }

    println!("3. 권장: (머리글, 접근 함수) 표로 고른 열만 내보내기");
    let header: Vec<&str> = COLUMNS.iter().map(|(name, _)| *name).collect();
    println!("    {}", header.join(","));
    for row in &rows {
        println!("    {}", export_with_columns(row, &COLUMNS));
    }
    // 출력:
    //     learner,progress,certificate
    //     김하나,80,false
    //     이두리,100,true

    // 비교를 위해: field()는 이름이 맞고 자기가 아는 필드일 때만 값을 돌려줍니다.
    // 1번 구역의 "?"는 실패를 알리는 에러가 아니라 이 None이 조용히 바뀐 것입니다.
    println!(
        "    (참고) field(\"course\") = {:?}",
        rows[0].field("course")
    );
    // 출력:     (참고) field("course") = Some("러스트 입문")
}
