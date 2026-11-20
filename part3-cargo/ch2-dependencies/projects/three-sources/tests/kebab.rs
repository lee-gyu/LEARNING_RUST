// 통합 테스트도 `[dependencies]`의 크레이트를 쓸 수 있습니다.
// `frame`이 쓰는 `unicode-width`는 여기서 쓸 수 없습니다. 직접 적은 의존성이 아니기 때문입니다.

use heck::ToKebabCase;

#[test]
fn kebab_then_box() {
    let name = "CargoBook".to_kebab_case();
    assert_eq!(name, "cargo-book");
    assert_eq!(
        frame::boxed(&name),
        "+------------+\n| cargo-book |\n+------------+"
    );
}
