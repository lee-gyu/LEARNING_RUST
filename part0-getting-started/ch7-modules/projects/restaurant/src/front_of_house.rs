// 모듈 front_of_house의 내용입니다 (src/front_of_house.rs).
// lib.rs의 `mod front_of_house;` 선언 덕분에 이 파일이 모듈 트리에 들어갑니다.
//
// 이 파일은 하위 모듈 두 개를 선언만 합니다. front_of_house의 하위 모듈이므로
// 컴파일러는 부모 이름을 딴 디렉터리 src/front_of_house/ 안에서 hosting.rs와 serving.rs를 찾습니다.
pub mod hosting;
pub mod serving;
