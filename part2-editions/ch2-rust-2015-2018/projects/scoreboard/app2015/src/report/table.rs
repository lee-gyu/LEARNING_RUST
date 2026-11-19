// 레코드 목록 전체를 그리는 모듈입니다(report의 자식 모듈).

// 2015: `use` 경로는 크레이트 루트에서 출발합니다. `record`는 크레이트 루트의 모듈입니다.
use record::Record;
// super::는 2015에서도 같은 뜻입니다(부모 모듈 report).
use super::Style;

// 2015 문법: &Style은 트레이트 객체에 대한 참조입니다.
#[allow(bare_trait_objects)]
pub fn render(records: &[Record], style: &Style) {
    for r in records {
        println!("  {}", style.line(&r.name, r.score));
    }
    let total: u32 = records.iter().map(|r| r.score).sum();
    let average = total / records.len() as u32;
    // 2015: ::로 시작하는 경로도 크레이트 루트에서 출발합니다.
    println!("  평균 {}점 ({})", average, ::report::grade(average));
}
