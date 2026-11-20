//! 의존성 그래프에 std가 섞여 들어왔는지 호스트에서 확인하는 검사용 크레이트입니다.
//!
//! `#[panic_handler]`는 `no_std` 프로그램이 패닉했을 때 할 일을 정하는 함수이고, 한 프로그램에 하나만
//! 있을 수 있습니다. 의존성 어딘가가 std를 연결하면 std가 가진 것과 겹쳐 E0152로 빌드가 실패하고,
//! note가 std를 끌어온 크레이트의 이름을 알려 줍니다. 자세한 것은 Part 10 `ch12-ffi-beneath-std`에서 다룹니다.
//!
//! 이 검사는 std 없는 타깃으로 크로스 컴파일하는 검사를 대신하지 못합니다(README 2.5절).

#![no_std]

// 검사할 크레이트를 이름으로 한 번씩 가리켜 연결합니다.
pub use sensor;
pub use tinyhex;

#[panic_handler]
fn on_panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
