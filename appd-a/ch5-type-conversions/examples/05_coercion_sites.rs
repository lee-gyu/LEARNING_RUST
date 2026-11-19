// 지침 5: 강제 변환은 받는 타입이 적힌 자리에서, 참조와 포인터의 모양을 바꾸는 데에만 기대합니다.
//
// 컴파일러가 알아서 해 주는 변환을 강제 변환(coercion)이라고 합니다. 받는 쪽 타입이 코드에 적혀 있는
// 자리(함수 인자, 타입을 적은 `let`, 반환 값, 구조체 필드 등)에서, 정해진 몇 가지 모양으로만 일어납니다.
// 1번 구역은 자주 만나는 강제 변환을 한곳에 모읍니다.
// 비권장: 제네릭 매개변수에도 역참조 강제 변환이 일어나리라 기대하고 `&String`을 넘깁니다.
//         타입 매개변수가 `String`으로 정해질 뿐 변환은 없어서, 기대와 다른 값을 잽니다.
// 권장:   제네릭에 넘길 때는 원하는 타입(`&str`)으로 직접 바꿔 넘깁니다.
// 3번 구역은 컨테이너 안쪽도 강제 변환이 일어나지 않는 자리임을 보여 줍니다.
//
// 실행: cargo run -p er-ch5-type-conversions --example 05_coercion_sites

use std::any::type_name;
use std::mem::size_of_val;
use std::rc::Rc;

trait Widget {
    fn draw(&self) -> String;
}

// `Widget`을 슈퍼트레이트로 가진 트레이트입니다. 업캐스트(`&dyn Clickable` → `&dyn Widget`)을
// 보이는 데만 쓰므로 메서드를 두지 않았습니다.
trait Clickable: Widget {}

struct Button(&'static str);

impl Widget for Button {
    fn draw(&self) -> String {
        format!("[{}]", self.0)
    }
}

impl Clickable for Button {}

struct Text(String);

impl Widget for Text {
    fn draw(&self) -> String {
        self.0.clone()
    }
}

struct Doc {
    title: String,
}

impl Doc {
    // 반환 값 자리: 본문의 `&self.title`은 `&String`이지만 반환 타입이 `&str`이라 강제 변환됩니다.
    fn title(&self) -> &str {
        &self.title
    }
}

fn shout(text: &str) -> String {
    text.to_uppercase()
}

fn total(xs: &[u32]) -> u32 {
    xs.iter().sum()
}

fn render(w: &dyn Widget) -> String {
    w.draw()
}

fn apply(f: fn(u32) -> u32, x: u32) -> u32 {
    f(x)
}

fn draw_all(widgets: &[Box<dyn Widget>]) -> String {
    widgets
        .iter()
        .map(|w| w.draw())
        .collect::<Vec<_>>()
        .join(" ")
}

/// 값이 차지하는 바이트 수와, 실제로 정해진 타입 매개변수를 돌려줍니다.
/// 어떤 값이든 재도록 제네릭으로 만들었습니다(`?Sized`로 `str` 같은 타입도 받습니다).
fn measure<T: ?Sized>(value: &T) -> (usize, &'static str) {
    (size_of_val(value), type_name::<T>())
}

fn main() {
    println!("1. 받는 타입이 적힌 자리에서 일어나는 강제 변환");
    let name = String::from("rust");
    let shared = Rc::new(String::from("shared"));
    let scores = vec![70, 85];
    let fixed = [1, 2, 3];
    let ok = Button("확인");
    println!("    &String → &str            : {}", shout(&name)); // 출력:     &String → &str            : RUST
    // 역참조 강제 변환은 필요한 만큼 이어서 일어납니다: &Rc<String> → &String → &str
    println!("    &Rc<String> → &str        : {}", shout(&shared)); // 출력:     &Rc<String> → &str        : SHARED
    println!("    &Vec<u32> → &[u32]        : {}", total(&scores)); // 출력:     &Vec<u32> → &[u32]        : 155
    println!("    &[u32; 3] → &[u32]        : {}", total(&fixed)); // 출력:     &[u32; 3] → &[u32]        : 6
    println!("    &Button → &dyn Widget     : {}", render(&ok)); // 출력:     &Button → &dyn Widget     : [확인]
    // 캡처하지 않는 클로저는 함수 포인터가 됩니다.
    println!("    클로저 → fn(u32) -> u32   : {}", apply(|x| x * 2, 21)); // 출력:     클로저 → fn(u32) -> u32   : 42
    // 타입을 적은 let 자리
    let mut count = 7;
    let view: &u32 = &mut count; // &mut u32 → &u32
    let boxed: Box<dyn Widget> = Box::new(Text(String::from("안내문"))); // Box<Text> → Box<dyn Widget>
    let clickable: &dyn Clickable = &ok;
    let as_widget: &dyn Widget = clickable; // &dyn Clickable → &dyn Widget (업캐스트, 1.86.0부터)
    println!("    &mut u32 → &u32           : {view}"); // 출력:     &mut u32 → &u32           : 7
    println!("    Box<Text> → Box<dyn ..>   : {}", boxed.draw()); // 출력:     Box<Text> → Box<dyn ..>   : 안내문
    println!("    &dyn Clickable → &dyn ..  : {}", as_widget.draw()); // 출력:     &dyn Clickable → &dyn ..  : [확인]
    let doc = Doc {
        title: String::from("변환 정리"),
    };
    println!("    반환 값 &String → &str    : {}", doc.title()); // 출력:     반환 값 &String → &str    : 변환 정리

    println!("2. 제네릭 매개변수에는 강제 변환이 없습니다");
    let short = String::from("hello");
    let long = String::from("강제 변환은 자리를 가립니다"); // 한글은 UTF-8로 글자당 3바이트
    // 비권장: `&String`을 넘기면 T = String으로 정해집니다. 잰 것은 글자가 아니라 String 자체(포인터,
    // 용량, 길이)의 크기라서 내용과 상관없이 같습니다(64비트 기준 24).
    println!("    비권장 &short        : {:?}", measure(&short)); // 출력:     비권장 &short        : (24, "alloc::string::String")
    println!("    비권장 &long         : {:?}", measure(&long)); // 출력:     비권장 &long         : (24, "alloc::string::String")
    // 권장: 원하는 타입을 직접 만들어 넘기면 T = str이 되어 글자의 바이트 수를 잽니다.
    println!("    권장 short.as_str()  : {:?}", measure(short.as_str())); // 출력:     권장 short.as_str()  : (5, "str")
    println!("    권장 long.as_str()   : {:?}", measure(long.as_str())); // 출력:     권장 long.as_str()   : (39, "str")

    println!("3. 컨테이너 안쪽에도 강제 변환이 없습니다");
    let buttons: Vec<Box<Button>> = vec![Box::new(Button("저장")), Box::new(Button("취소"))];
    // `draw_all(&buttons)`는 E0308입니다(compile_fail/e0308_no_coercion_inside_vec.rs).
    // 원소를 하나씩 바꿔 새 Vec을 만듭니다. 원소 자리에서는 `as`로 강제 변환을 일으킬 수 있습니다.
    let converted: Vec<Box<dyn Widget>> =
        buttons.into_iter().map(|b| b as Box<dyn Widget>).collect();
    println!("    원소를 바꿔 만든 Vec : {}", draw_all(&converted)); // 출력:     원소를 바꿔 만든 Vec : [저장] [취소]
    // 처음부터 받는 타입을 적어 두면 원소 하나하나가 강제 변환 자리가 됩니다.
    let mixed: Vec<Box<dyn Widget>> = vec![
        Box::new(Button("열기")),
        Box::new(Text(String::from("파일 없음"))),
    ];
    println!("    처음부터 dyn인 Vec   : {}", draw_all(&mixed)); // 출력:     처음부터 dyn인 Vec   : [열기] 파일 없음
}
