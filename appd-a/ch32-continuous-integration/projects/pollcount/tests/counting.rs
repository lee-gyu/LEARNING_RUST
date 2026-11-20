//! 공개 API만으로 쓰는 통합 테스트입니다.

use pollcount::Votes;

#[test]
fn counts_survey_answers() {
    let mut t = Votes::new();
    t.extend(["사과", " 배", "사과 ", "", "귤", "사과"]);
    assert_eq!(t.total(), 5);
    assert_eq!(t.len(), 3);
    assert_eq!(t.top(2), vec![("사과", 3), ("귤", 1)]);
}

#[test]
fn empty_input_gives_no_votes() {
    let t: Votes = " , ,".parse().unwrap();
    assert!(t.is_empty());
    assert_eq!(t.top(5), Vec::<(&str, u64)>::new());
}

#[cfg(all(feature = "csv", feature = "json"))]
#[test]
fn both_exports_list_the_same_order() {
    let t: Votes = "b, a, b".parse().unwrap();
    assert!(t.to_csv().starts_with("item,count\nb,2\na,1"));
    assert!(t.to_json().starts_with(r#"[{"item":"b","count":2}"#));
}
