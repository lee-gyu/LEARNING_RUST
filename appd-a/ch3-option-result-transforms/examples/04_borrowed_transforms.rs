// 이 예제가 보여 주는 것: 참조 뒤의 Option에 변환 메서드나 ?를 쓸 때는 as_ref·as_deref로 참조를 담은 Option을 만든다
// 실행: cargo run -q -p er-ch3-option-result-transforms --example 04_borrowed_transforms
//
// match를 변환 메서드로 바꿀 때 가장 자주 부딪히는 차이입니다.
// - match &self.field { Some(x) => .. }는 참조를 조건식으로 받으므로 x가 저절로 참조로 바인딩됩니다.
// - 변환 메서드(map, unwrap_or, ...)와 ?는 Option을 self로, 즉 통째로 가져갑니다.
//   &self로 빌린 구조체의 필드는 가져갈 수 없으므로, as_ref()로 Option<&T>를 만들어 그것을 가져가게 합니다.

struct Parcel {
    tracking_no: Option<String>, // 발송 전에는 None
    weight_g: Option<u32>,       // 아직 재지 않았으면 None
}

impl Parcel {
    // match판: &self.tracking_no를 조건식으로 주면 no는 &String입니다. as_ref가 필요 없습니다.
    // clippy는 이 match를 map으로 바꾸라고 하면서, 제안하는 코드에 as_ref()를 넣어 줍니다(manual_map).
    #[expect(clippy::manual_map, reason = "비권장 형태를 비교하려고 남겨 둡니다")]
    fn tracking_len_match(&self) -> Option<usize> {
        match &self.tracking_no {
            Some(no) => Some(no.chars().count()),
            None => None,
        }
    }

    // 변환 메서드판: as_ref()가 &Option<String>을 Option<&String>으로 바꿉니다.
    // map이 가져가는 것은 이 새 Option이므로 필드는 그대로 남습니다.
    fn tracking_len(&self) -> Option<usize> {
        self.tracking_no.as_ref().map(|no| no.chars().count())
    }

    // 비권장: as_ref 없이 unwrap_or를 쓰면 E0507이고, 컴파일러는 clone()을 제안합니다.
    // 제안대로 하면 컴파일은 되지만 부를 때마다 문자열을 새로 할당해 복사합니다.
    fn label_clone(&self) -> String {
        self.tracking_no
            .clone()
            .unwrap_or_else(|| String::from("발송 전"))
    }

    // 권장: as_deref()는 Option<String>을 빌려 Option<&str>을 만듭니다. 복사 없이 필드 안의 문자열을 가리킵니다.
    fn label(&self) -> &str {
        self.tracking_no.as_deref().unwrap_or("발송 전")
    }

    // ? 앞에도 같은 규칙이 적용됩니다. self.tracking_no?는 E0507입니다(compile_fail 참고).
    // 송장 번호의 앞 두 글자를 택배사 코드로 봅니다. get(..2)은 범위를 벗어나면 패닉하지 않고 None입니다.
    fn carrier(&self) -> Option<&str> {
        let no = self.tracking_no.as_deref()?;
        no.get(..2)
    }

    // 따르지 않아도 되는 경우 1: Option<u32>처럼 안의 타입이 Copy면 Option도 Copy입니다.
    // map이 가져가는 것은 복사본이므로 as_ref가 필요 없습니다.
    fn weight_kg(&self) -> Option<f64> {
        self.weight_g.map(|g| f64::from(g) / 1000.0)
    }

    // 따르지 않아도 되는 경우 2: self를 받는(소비하는) 메서드는 필드를 가져가도 됩니다.
    // 여기서 as_ref를 쓰면 오히려 to_string() 같은 복사가 따로 필요해집니다.
    fn into_label(self) -> String {
        self.tracking_no.unwrap_or_else(|| String::from("발송 전"))
    }
}

fn main() {
    let sent = Parcel {
        tracking_no: Some(String::from("CJ2026111901")),
        weight_g: Some(1250),
    };
    let waiting = Parcel {
        tracking_no: None,
        weight_g: None,
    };

    println!("== 1. match는 참조를 받고, 변환 메서드는 as_ref가 필요합니다 ==");
    for (name, p) in [("발송", &sent), ("대기", &waiting)] {
        println!(
            "{name}: match판 {:?}, as_ref판 {:?}",
            p.tracking_len_match(),
            p.tracking_len()
        );
    }
    // 출력:
    // 발송: match판 Some(12), as_ref판 Some(12)
    // 대기: match판 None, as_ref판 None

    println!();
    println!("== 2. 값을 꺼낼 때: clone() 대신 as_deref() ==");
    for (name, p) in [("발송", &sent), ("대기", &waiting)] {
        println!(
            "{name}: clone판 {:?}, as_deref판 {:?}",
            p.label_clone(),
            p.label()
        );
    }
    // 같은 문자열이지만 clone판은 새 버퍼, as_deref판은 필드 안의 버퍼를 가리킵니다.
    // 주소 값 자체는 실행마다 다르므로 출력하지 않고, 필드의 버퍼와 같은 주소인지만 비교합니다.
    let field_ptr = sent.tracking_no.as_deref().map(str::as_ptr);
    let cloned = sent.label_clone();
    let borrowed = sent.label();
    println!(
        "필드와 같은 버퍼인가: clone판 {}, as_deref판 {}",
        field_ptr == Some(cloned.as_ptr()),
        field_ptr == Some(borrowed.as_ptr())
    );
    // 출력:
    // 발송: clone판 "CJ2026111901", as_deref판 "CJ2026111901"
    // 대기: clone판 "발송 전", as_deref판 "발송 전"
    // 필드와 같은 버퍼인가: clone판 false, as_deref판 true

    println!();
    println!("== 3. ? 앞에도 as_deref() ==");
    println!("발송: {:?}, 대기: {:?}", sent.carrier(), waiting.carrier());
    // 출력:
    // 발송: Some("CJ"), 대기: None

    println!();
    println!("== 4. as_ref가 필요 없는 경우 ==");
    println!(
        "무게(Copy인 Option<u32>): 발송 {:?}, 대기 {:?}",
        sent.weight_kg(),
        waiting.weight_kg()
    );
    // 빌려서 쓰는 일이 모두 끝난 뒤에 self를 소비하는 메서드를 부릅니다.
    println!("into_label: {:?}", sent.into_label());
    // 출력:
    // 무게(Copy인 Option<u32>): 발송 Some(1.25), 대기 None
    // into_label: "CJ2026111901"
}
