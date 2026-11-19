//! 지침 2: 패닉판을 줄 때는 `Result`판(또는 `Option`판)을 함께 주고, 패닉은 버그를 알리는 데 쓴다.
//!
//! 글자로 그림을 그리는 캔버스 라이브러리를 두 판으로 만듭니다.
//! - `canvas_v1`: 패닉판만 있고, 패닉 위치가 라이브러리 안을 가리킵니다.
//! - `canvas`: `#[track_caller]`를 붙인 패닉판 `plot`·`Index`와, 실패를 돌려주는 `try_plot`·`get`.
//!
//! 실행: cargo run -p er-ch18-dont-panic --example 02_panic_flavor_pairs
//!
//! 패닉이 난 위치(파일:줄)를 표준 출력에서 비교하려고 패닉 훅을 바꿉니다. 바꾼 훅은 메시지와
//! 위치를 기록만 하고 출력하지 않으므로, 이 예제는 표준 에러에 패닉 메시지를 내지 않습니다.

use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::{Mutex, PoisonError};

/// 비권장: 패닉판만 있는 캔버스.
mod canvas_v1 {
    pub struct Canvas {
        width: usize,
        height: usize,
        cells: Vec<char>,
    }

    impl Canvas {
        pub fn new(width: usize, height: usize) -> Canvas {
            Canvas {
                width,
                height,
                cells: vec!['.'; width * height],
            }
        }

        /// (x, y)에 글자를 찍습니다. 캔버스 밖이면 패닉합니다.
        pub fn plot(&mut self, x: usize, y: usize, ch: char) {
            if x >= self.width || y >= self.height {
                let (w, h) = (self.width, self.height);
                panic!("{w}x{h} 캔버스 밖의 좌표입니다: ({x}, {y})");
            }
            self.cells[y * self.width + x] = ch;
        }
    }
}

/// 권장: 패닉판과 실패를 돌려주는 판을 짝으로 주는 캔버스.
mod canvas {
    use std::error::Error;
    use std::fmt;
    use std::ops::Index;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OutOfCanvas {
        pub x: usize,
        pub y: usize,
    }

    impl fmt::Display for OutOfCanvas {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "캔버스 밖의 좌표입니다: ({}, {})", self.x, self.y)
        }
    }

    impl Error for OutOfCanvas {}

    /// 불변식: `cells.len() == width * height`
    pub struct Canvas {
        width: usize,
        height: usize,
        cells: Vec<char>,
    }

    impl Canvas {
        pub fn new(width: usize, height: usize) -> Canvas {
            Canvas {
                width,
                height,
                cells: vec!['.'; width * height],
            }
        }

        /// 좌표가 캔버스 안이면 `cells`의 위치를 돌려줍니다. 경계 검사는 이곳 한 군데에서 합니다.
        fn offset(&self, x: usize, y: usize) -> Option<usize> {
            (x < self.width && y < self.height).then(|| y * self.width + x)
        }

        /// (x, y)에 글자를 찍습니다.
        ///
        /// # Panics
        ///
        /// (x, y)가 캔버스 밖이면 패닉합니다. 좌표가 사용자 입력처럼 바깥에서 온다면
        /// [`Canvas::try_plot`]을 쓰세요.
        #[track_caller] // 패닉 위치를 이 함수의 본문이 아니라 부른 줄로 보고합니다.
        pub fn plot(&mut self, x: usize, y: usize, ch: char) {
            if let Err(e) = self.try_plot(x, y, ch) {
                panic!("{e}");
            }
        }

        /// (x, y)에 글자를 찍습니다.
        ///
        /// # Errors
        ///
        /// (x, y)가 캔버스 밖이면 아무것도 바꾸지 않고 `OutOfCanvas`를 돌려줍니다.
        pub fn try_plot(&mut self, x: usize, y: usize, ch: char) -> Result<(), OutOfCanvas> {
            let i = self.offset(x, y).ok_or(OutOfCanvas { x, y })?;
            self.cells[i] = ch; // offset이 범위를 확인했으므로 패닉하지 않습니다.
            Ok(())
        }

        /// (x, y)의 글자. 캔버스 밖이면 `None`입니다. 패닉판은 `canvas[(x, y)]`입니다.
        pub fn get(&self, x: usize, y: usize) -> Option<char> {
            self.offset(x, y).map(|i| self.cells[i])
        }

        /// y번째 행을 문자열로 돌려줍니다. 캔버스 밖의 행이면 `None`입니다.
        pub fn row(&self, y: usize) -> Option<String> {
            if y >= self.height {
                return None; // 부르는 쪽이 일으킬 수 있는 실패 → 값으로 알림
            }
            let start = y * self.width;
            let cells = self
                .cells
                .get(start..start + self.width)
                // 이 expect는 Canvas 자신의 코드가 틀렸을 때만 실패합니다(불변식 위반).
                // 부르는 쪽이 고칠 수 있는 일이 아니므로 Result판을 두지 않습니다.
                .expect("Canvas의 cells는 width * height칸이어야 합니다");
            Some(cells.iter().collect())
        }

        /// 버그를 흉내 내는 메서드: '.'로 채워야 할 칸을 통째로 비워 불변식을 깨뜨립니다.
        pub fn buggy_clear(&mut self) {
            self.cells.clear();
        }
    }

    impl Index<(usize, usize)> for Canvas {
        type Output = char;

        /// # Panics
        ///
        /// (x, y)가 캔버스 밖이면 패닉합니다. 패닉하지 않는 판은 [`Canvas::get`]입니다.
        fn index(&self, (x, y): (usize, usize)) -> &char {
            // Index::index는 트레이트 선언에 #[track_caller]가 붙어 있어서
            // 구현에 따로 붙이지 않아도 패닉 위치가 canvas[(x, y)]를 쓴 줄이 됩니다.
            match self.offset(x, y) {
                Some(i) => &self.cells[i],
                None => panic!("{}", OutOfCanvas { x, y }),
            }
        }
    }
}

/// 마지막 패닉의 (메시지, "파일:줄")
static LAST_PANIC: Mutex<Option<(String, String)>> = Mutex::new(None);

fn install_recording_hook() {
    panic::set_hook(Box::new(|info| {
        let message = info.payload_as_str().unwrap_or("(문자열이 아닌 페이로드)");
        let location = info
            .location()
            .map(|l| {
                let file = Path::new(l.file()).file_name().unwrap_or_default();
                format!("{}:{}", file.display(), l.line())
            })
            .unwrap_or_default();
        let mut last = LAST_PANIC.lock().unwrap_or_else(PoisonError::into_inner);
        *last = Some((message.to_string(), location));
    }));
}

/// 클로저를 실행하고, 패닉하면 훅이 기록한 (메시지, 위치)를 돌려줍니다.
fn attempt<R>(f: impl FnOnce() -> R) -> Result<R, (String, String)> {
    panic::catch_unwind(AssertUnwindSafe(f)).map_err(|_| {
        let mut last = LAST_PANIC.lock().unwrap_or_else(PoisonError::into_inner);
        last.take().unwrap_or_default()
    })
}

fn report_panic(label: &str, result: Result<(), (String, String)>, call_line: u32) {
    match result {
        Ok(()) => println!("    {label} → 성공"),
        Err((message, location)) => {
            println!("    {label} → 패닉: {message}");
            println!("      패닉 위치: {location} (부른 줄: {call_line}행)");
        }
    }
}

fn main() {
    install_recording_hook();

    println!("1. 비권장: 패닉판만 있는 라이브러리");
    let mut old = canvas_v1::Canvas::new(8, 4);
    let call_line = line!() + 1;
    let result = attempt(|| old.plot(9, 1, '#'));
    report_panic("plot(9, 1)", result, call_line);
    // 출력: 패닉 위치가 canvas_v1::Canvas::plot 안의 panic! 줄이라 어느 호출이 틀렸는지 모릅니다.

    println!("2. 권장: #[track_caller]를 붙인 패닉판과 실패를 돌려주는 판을 함께 줌");
    let mut canvas = canvas::Canvas::new(8, 4);
    let call_line = line!() + 1;
    let result = attempt(|| canvas.plot(9, 1, '#'));
    report_panic("plot(9, 1)", result, call_line);
    let call_line = line!() + 1;
    let result = attempt(|| canvas[(9, 1)]).map(|_| ());
    report_panic("canvas[(9, 1)]", result, call_line);
    // 출력: 두 패닉 모두 위치가 부른 줄과 같습니다.

    canvas.plot(2, 1, '#'); // 코드에 적은 좌표: 틀리면 버그이므로 패닉판으로 충분합니다.
    println!(
        "    get(9, 1) = {:?}, canvas[(2, 1)] = {:?}",
        canvas.get(9, 1),
        canvas[(2, 1)]
    );
    // 출력: get(9, 1) = None, canvas[(2, 1)] = '#'

    // 대각선의 시작점과 길이가 사용자 입력이라고 가정합니다. 캔버스 밖으로 나가는 것은
    // 버그가 아니라 일어날 수 있는 일이므로 try_plot으로 잘라 냅니다.
    let (start_x, length) = (4, 6);
    let mut skipped = 0;
    for k in 0..length {
        if canvas.try_plot(start_x + k, k, '\\').is_err() {
            skipped += 1;
        }
    }
    println!(
        "    try_plot으로 대각선 긋기: 찍은 점 {}개, 캔버스 밖이라 건너뛴 점 {skipped}개",
        length - skipped
    );
    for y in 0..4 {
        if let Some(row) = canvas.row(y) {
            println!("      {row}");
        }
    }
    // 출력: 찍은 점 4개, 건너뛴 점 2개

    println!("3. 불변식이 깨진 내부 버그에는 패닉이 맞음");
    canvas.buggy_clear();
    let call_line = line!() + 1;
    let result = attempt(|| canvas.row(0)).map(|_| ());
    report_panic("row(0)", result, call_line);
    // 출력: 패닉 위치가 Canvas::row 안의 expect 줄입니다. 고칠 곳은 부른 쪽이 아니라 Canvas입니다.
}
