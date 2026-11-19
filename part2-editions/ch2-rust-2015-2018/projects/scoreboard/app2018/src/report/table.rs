// 레코드 목록 전체를 그리는 모듈입니다(report의 자식 모듈).

// 2018: 크레이트 루트에서 출발하려면 crate::로 시작합니다.
use crate::record::Record;
// super::는 2015에서도 같은 뜻입니다(부모 모듈 report).
use super::Style;

// 2018: &dyn Style은 트레이트 객체에 대한 참조입니다.
pub fn render(records: &[Record], style: &dyn Style) {
    for r in records {
        println!("  {}", style.line(&r.name, r.score));
    }
    let total: u32 = records.iter().map(|r| r.score).sum();
    let average = total / records.len() as u32;
    // 2018: ::로 시작하는 경로는 외부 크레이트만 가리키므로, 크레이트 루트는 crate::로 씁니다.
    println!("  평균 {}점 ({})", average, crate::report::grade(average));
}
