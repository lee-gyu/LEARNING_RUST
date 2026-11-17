// 모듈 front_of_house::serving의 내용입니다 (src/front_of_house/serving.rs).

// 다른 파일에 있는 항목도 파일 경로가 아니라 모듈 트리의 경로로 가리킵니다.
// Breakfast는 src/back_of_house.rs에 있지만, 경로는 crate::back_of_house::Breakfast입니다.
use crate::back_of_house::Breakfast;

pub fn take_order(name: &str, toast: &str) -> Breakfast {
    println!("  [{}] {name}님의 주문: {toast} 토스트", module_path!());
    Breakfast::summer(toast)
}

pub fn serve_order(meal: &Breakfast) {
    println!(
        "  [{}] {} 토스트를 내갑니다 (곁들임: {})",
        module_path!(),
        meal.toast,
        meal.fruit()
    );
}
