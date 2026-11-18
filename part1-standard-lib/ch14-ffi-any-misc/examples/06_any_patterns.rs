// Any를 정당하게 쓰는 두 가지 모양과, 열거형이 더 나은 경우.
// (1) 타입마다 값을 하나씩 담는 저장소(타입 맵), (2) 자기 트레이트 객체를 dyn Any로 업캐스트해
// 특정 타입만 골라내기, (3) 같은 일을 열거형으로 했을 때의 비교.
//
// 실행: cargo run -p p1-ch14-ffi-any-misc --example 06_any_patterns

use std::any::{Any, TypeId};
use std::collections::HashMap;

// ── 1. 타입 맵 ──────────────────────────────────────────────

/// 타입마다 값을 하나씩 담는 저장소입니다. 키는 값의 TypeId입니다.
/// 웹 프레임워크가 요청에 붙이는 "확장 데이터"처럼, 저장소를 만드는 쪽이
/// 어떤 타입이 들어올지 미리 알 수 없을 때 씁니다.
#[derive(Default)]
struct TypeMap {
    values: HashMap<TypeId, Box<dyn Any>>,
}

impl TypeMap {
    /// 넣고, 같은 타입의 값이 이미 있었다면 그 값을 돌려줍니다.
    fn insert<T: Any>(&mut self, value: T) -> Option<T> {
        let old = self.values.insert(TypeId::of::<T>(), Box::new(value))?;
        // TypeId::of::<T>() 칸에는 insert가 T만 넣으므로 이 다운캐스트는 실패하지 않습니다.
        // 이 불변식은 타입이 아니라 이 구조체의 코드가 지킵니다.
        old.downcast::<T>().ok().map(|boxed| *boxed)
    }

    fn get<T: Any>(&self) -> Option<&T> {
        // values.get은 Option<&Box<dyn Any>>입니다. downcast_ref는 dyn Any의 고유 메서드라서
        // 자동 역참조가 상자 안의 dyn Any까지 들어가 찾아 줍니다.
        self.values.get(&TypeId::of::<T>())?.downcast_ref::<T>()
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.values.get_mut(&TypeId::of::<T>())?.downcast_mut::<T>()
    }
}

struct Config {
    verbose: bool,
}

struct RequestId(u64);

// ── 2. 트레이트 객체 업캐스트 ───────────────────────────────

// Any를 슈퍼트레이트로 두면 dyn Widget을 dyn Any로 바꿀 수 있습니다(트레이트 객체 업캐스트, 1.86.0).
trait Widget: Any {
    fn name(&self) -> String;
}

struct Button {
    label: String,
    clicks: u32,
}

struct Slider {
    value: u8,
}

impl Widget for Button {
    fn name(&self) -> String {
        format!("버튼 '{}'", self.label)
    }
}

impl Widget for Slider {
    fn name(&self) -> String {
        format!("슬라이더 {}", self.value)
    }
}

// ── 3. 열거형으로 같은 일 ───────────────────────────────────

enum WidgetKind {
    Button(Button),
    Slider(Slider),
}

fn main() {
    println!("1. 타입 맵: 타입이 곧 키입니다");
    let mut ext = TypeMap::default();
    ext.insert(Config { verbose: true });
    ext.insert(RequestId(7));
    ext.insert(vec![String::from("인증 통과")]);
    println!(
        "    Config.verbose = {:?}, RequestId = {:?}",
        ext.get::<Config>().map(|c| c.verbose),
        ext.get::<RequestId>().map(|r| r.0)
    );
    // 출력:     Config.verbose = Some(true), RequestId = Some(7)
    // RequestId 안에 u64가 있어도 u64로는 찾을 수 없습니다. 키는 정확한 타입입니다.
    println!("    get::<u64>() = {:?}", ext.get::<u64>()); // 출력:     get::<u64>() = None
    if let Some(log) = ext.get_mut::<Vec<String>>() {
        log.push(String::from("권한 확인"));
    }
    println!("    로그 = {:?}", ext.get::<Vec<String>>()); // 출력:     로그 = Some(["인증 통과", "권한 확인"])
    let old = ext.insert(RequestId(8));
    println!(
        "    같은 타입을 다시 넣으면 옛 값을 돌려줍니다: {:?} → {:?}",
        old.map(|r| r.0),
        ext.get::<RequestId>().map(|r| r.0)
    );
    // 출력:     같은 타입을 다시 넣으면 옛 값을 돌려줍니다: Some(7) → Some(8)

    println!("2. dyn Widget → dyn Any 업캐스트로 특정 타입만 골라내기");
    let mut widgets: Vec<Box<dyn Widget>> = vec![
        Box::new(Button {
            label: String::from("저장"),
            clicks: 3,
        }),
        Box::new(Slider { value: 40 }),
        Box::new(Button {
            label: String::from("취소"),
            clicks: 0,
        }),
    ];
    // 모든 위젯에 공통인 일은 트레이트 메서드로 합니다. 다운캐스트가 필요 없습니다.
    let names: Vec<String> = widgets.iter().map(|w| w.name()).collect();
    println!("    {names:?}"); // 출력:     ["버튼 '저장'", "슬라이더 40", "버튼 '취소'"]
    // Button에만 있는 필드(clicks)가 필요하면 &dyn Widget을 &dyn Any로 올린 뒤 다운캐스트합니다.
    // &**w: &Box<dyn Widget> → Box<dyn Widget> → dyn Widget을 빌린 &dyn Widget.
    // 상자(&Box)째로 올리면 상자 자체가 Any 값이 되므로(예제 05) 상자 안을 빌려야 합니다.
    let total_clicks: u32 = widgets
        .iter()
        .filter_map(|w| {
            let any: &dyn Any = &**w; // 업캐스트
            any.downcast_ref::<Button>()
        })
        .map(|b| b.clicks)
        .sum();
    println!("    버튼 클릭 합계: {total_clicks}"); // 출력:     버튼 클릭 합계: 3
    // &mut dyn Widget도 &mut dyn Any로 올려 downcast_mut할 수 있습니다.
    for w in widgets.iter_mut() {
        let any: &mut dyn Any = &mut **w;
        if let Some(slider) = any.downcast_mut::<Slider>() {
            slider.value = 100;
        }
    }
    // Box<dyn Widget>도 Box<dyn Any>로 올려 소유권째 꺼낼 수 있습니다.
    let second: Box<dyn Any> = widgets.remove(1);
    match second.downcast::<Slider>() {
        Ok(slider) => println!("    꺼낸 슬라이더 값: {}", slider.value),
        Err(_) => println!("    슬라이더가 아닙니다"),
    }
    // 출력:     꺼낸 슬라이더 값: 100

    println!("3. 종류가 정해져 있다면 열거형이 낫습니다");
    let kinds = [
        WidgetKind::Button(Button {
            label: String::from("저장"),
            clicks: 3,
        }),
        WidgetKind::Slider(Slider { value: 40 }),
    ];
    // match는 모든 배리언트를 다뤘는지 컴파일러가 검사합니다. 배리언트를 하나 더 만들면
    // 이 match가 컴파일 에러가 되어 고칠 곳을 알려 줍니다. 다운캐스트의 else 갈래는 그러지 못합니다.
    let total: u32 = kinds
        .iter()
        .map(|k| match k {
            WidgetKind::Button(b) => b.clicks,
            WidgetKind::Slider(_) => 0,
        })
        .sum();
    let labels: Vec<String> = kinds
        .iter()
        .map(|k| match k {
            WidgetKind::Button(b) => b.label.clone(),
            WidgetKind::Slider(s) => format!("슬라이더 {}", s.value),
        })
        .collect();
    println!("    클릭 합계 {total}, 이름 {labels:?}"); // 출력:     클릭 합계 3, 이름 ["저장", "슬라이더 40"]
}
