// 2021 에디션부터 클로저는 변수 전체가 아니라 본문이 실제로 쓰는 경로(예: t.0, job.name)만 캡처합니다.
// 그 덕분에 새로 컴파일되는 코드와, 값이 버려지는(drop) 시점이 달라지는 코드를 봅니다.
// 각 구역 주석의 "2018에서는"은 같은 코드를 2018 에디션으로 컴파일했을 때의 결과입니다.
//
// 실행: cargo run -p p2-ch3-rust-2021 --example 03_disjoint_capture

use std::rc::Rc;
use std::thread;

/// 버려질 때 이름을 출력해서 drop 시점을 눈으로 확인하게 해 줍니다.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    drop: {}", self.0);
    }
}

struct Job {
    name: String,
    // Rc는 Send가 아니므로 Job도 Send가 아닙니다.
    cache: Rc<Vec<u8>>,
}

fn main() {
    println!("1. 서로 다른 필드를 동시에 빌리기");
    let mut pair = (String::from("왼쪽"), String::from("오른쪽"));
    // 클로저는 pair.0만 가변으로 빌립니다. 그래서 클로저가 살아 있는 동안에도 pair.1을 바꿀 수 있습니다.
    // 2018에서는: 클로저가 pair 전체를 빌리므로 다음 줄에서 E0499입니다.
    let mut push_left = || pair.0.push('!');
    pair.1.push('?');
    push_left();
    println!("    {pair:?}");
    // 출력:     ("왼쪽!", "오른쪽?")

    println!("2. 필드 하나를 옮긴 뒤 다른 필드를 쓰는 클로저");
    let point = (String::from("x좌표"), String::from("y좌표"));
    let moved_x = point.0; // point.0만 옮겨 갔습니다(부분 이동).
    // 클로저는 point.1만 빌리므로 문제가 없습니다.
    // 2018에서는: 클로저가 point 전체를 빌리려 하므로 E0382(부분적으로 이동된 값)입니다.
    let show_y = || println!("    남은 필드: {}", point.1);
    show_y();
    // 출력:     남은 필드: y좌표
    println!("    옮겨 간 필드: {moved_x}");
    // 출력:     옮겨 간 필드: x좌표

    println!("3. Send인 필드만 다른 스레드로 보내기");
    let job = Job {
        name: String::from("백업"),
        cache: Rc::new(vec![1, 2, 3]),
    };
    // move 클로저가 가져가는 것은 job.name(String, Send)뿐이라서 클로저도 Send입니다.
    // 2018에서는: job 전체(Rc 필드 포함)를 가져가므로 E0277 `Rc<Vec<u8>>` cannot be sent입니다.
    let handle = thread::spawn(move || format!("{} 작업을 다른 스레드에서 시작", job.name));
    println!("    {}", handle.join().unwrap());
    // 출력:     백업 작업을 다른 스레드에서 시작
    // job.cache는 옮겨 가지 않았으므로 여기서 계속 씁니다.
    println!("    cache는 그대로: {:?}", job.cache);
    // 출력:     cache는 그대로: [1, 2, 3]

    println!("4. 필드 하나만 캡처하면 버려지는 시점이 달라집니다");
    {
        let t = (Noisy("t.0"), Noisy("t.1"));
        {
            // move 클로저가 가져가는 것은 t.0뿐입니다.
            let _c = move || {
                let _first = &t.0;
            };
            println!("    안쪽 블록 끝");
        } // _c와 함께 t.0이 버려집니다.
        println!("    바깥 블록 끝");
    } // 남아 있던 t.1은 여기서 버려집니다.
    // 출력:     안쪽 블록 끝
    // 출력:     drop: t.0
    // 출력:     바깥 블록 끝
    // 출력:     drop: t.1
    // 2018에서는: 클로저가 t 전체를 가져가므로 "안쪽 블록 끝" 바로 뒤에 t.0과 t.1이 함께 버려집니다.

    println!("5. let _ = &t; 로 변수 전체를 캡처하게 만들기");
    {
        let t = (Noisy("u.0"), Noisy("u.1"));
        {
            let _c = move || {
                // &t는 계산해야 하는 표현식이라서 t 전체가 캡처됩니다.
                // (let _ = t; 는 아무것도 읽지 않으므로 캡처를 일으키지 않습니다.)
                let _ = &t;
                let _first = &t.0;
            };
            println!("    안쪽 블록 끝");
        } // _c와 함께 t 전체가 버려집니다.
        println!("    바깥 블록 끝");
    }
    // 출력:     안쪽 블록 끝
    // 출력:     drop: u.0
    // 출력:     drop: u.1
    // 출력:     바깥 블록 끝
}
