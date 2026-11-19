//! planner 0.3.0: 알림음을 더하려고 ring-tone을 들였는데, chime-sys를 이미 다른 버전으로 쓰고 있습니다.

fn main() {
    println!("직접 쓰는 chime-sys: {}", chime_sys::VERSION);
    println!("ring-tone의 알림음: {}", ring_tone::tone());
}
