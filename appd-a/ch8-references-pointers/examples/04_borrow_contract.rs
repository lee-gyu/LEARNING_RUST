// 지침 4: `Borrow<Q>`를 구현할 때는 `Eq`·`Hash`·`Ord`가 빌린 형태(`Q`)와 같은 답을 내게 합니다.
//
// `HashMap::get(&Q)`는 `K: Borrow<Q>`를 요구하고, 질의 `q`의 해시로 자리를 찾은 뒤
// `k.borrow() == q`로 키를 확인합니다. 즉 "키 타입끼리 같으면 빌린 형태도 같고 해시도 같다"고
// 믿습니다. 이 약속을 깨는 `Borrow` 구현은 컴파일은 되지만, 같은 키를 어떤 타입으로 찾느냐에 따라
// 맵이 서로 모순된 답을 내게 만듭니다.
//
// 실행: cargo run -p er-ch8-references-pointers --example 04_borrow_contract

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// ---------- 비권장: 비교 기준과 빌려주는 문자열이 다릅니다 ----------

/// 공항 코드를 입력한 그대로 보관하고, 비교·해시는 대소문자를 무시합니다.
#[derive(Debug)]
struct LooseCode(String);

impl LooseCode {
    fn new(raw: &str) -> Self {
        LooseCode(raw.to_string())
    }
}

impl PartialEq for LooseCode {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

impl Eq for LooseCode {}

impl Hash for LooseCode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Eq와 맞추려고 대문자로 바꾼 문자열의 해시를 씁니다. 키 타입끼리는 일관됩니다.
        self.0.to_ascii_uppercase().hash(state);
    }
}

/// 약속 위반: LooseCode로는 "icn"과 "ICN"이 같은데, 빌려준 str로는 다릅니다.
impl Borrow<str> for LooseCode {
    fn borrow(&self) -> &str {
        &self.0
    }
}

// ---------- 권장: 만들 때 정규화하고, 저장한 문자열 그대로 비교합니다 ----------

/// 저장된 문자열이 곧 비교 기준이므로, derive한 Eq·Hash·Ord가 str의 결과와 같습니다.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct AirportCode(String);

impl AirportCode {
    fn new(raw: &str) -> Self {
        AirportCode(raw.trim().to_ascii_uppercase())
    }
}

impl Borrow<str> for AirportCode {
    fn borrow(&self) -> &str {
        &self.0
    }
}

// ---------- 다른 선택: Borrow를 구현하지 않습니다 ----------

/// 바깥에 문자열을 보여 주기만 하고(AsRef), 같은 키라고 약속하지는 않습니다(Borrow 없음).
#[derive(Debug, PartialEq, Eq, Hash)]
struct StrictCode(String);

impl StrictCode {
    fn new(raw: &str) -> Self {
        StrictCode(raw.trim().to_ascii_uppercase())
    }
}

impl AsRef<str> for StrictCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn main() {
    // 사용자가 제각각 입력한 코드로 맵을 만듭니다.
    let entries = [("icn", "인천"), ("Gmp", "김포"), ("CJU", "제주")];

    println!("1. 비권장: Eq·Hash와 어긋나는 Borrow<str>");
    let mut loose: HashMap<LooseCode, &str> = entries
        .iter()
        .map(|&(code, name)| (LooseCode::new(code), name))
        .collect();
    println!(
        "    LooseCode로 \"ICN\" 찾기: {:?}, &str로 \"ICN\" 찾기: {:?}",
        loose.get(&LooseCode::new("ICN")),
        loose.get("ICN")
    );
    // 출력:     LooseCode로 "ICN" 찾기: Some("인천"), &str로 "ICN" 찾기: None
    println!(
        "    &str로 \"CJU\" 찾기: {:?}, \"cju\" 찾기: {:?}",
        loose.get("CJU"),
        loose.get("cju")
    );
    // 출력:     &str로 "CJU" 찾기: Some("제주"), "cju" 찾기: None
    // "ICN이 없으면 새로 추가"하는 코드는 기존 항목을 덮어씁니다.
    if !loose.contains_key("ICN") {
        loose.insert(LooseCode::new("ICN"), "인천(신규 등록)");
    }
    println!(
        "    ICN을 새로 추가했는데 항목 수 {}, icn의 값: {:?}",
        loose.len(),
        loose.get(&LooseCode::new("icn"))
    );
    // 출력:     ICN을 새로 추가했는데 항목 수 3, icn의 값: Some("인천(신규 등록)")

    println!("2. 권장: 만들 때 정규화해 Borrow의 약속을 지킵니다");
    let mut strict: HashMap<AirportCode, &str> = entries
        .iter()
        .map(|&(code, name)| (AirportCode::new(code), name))
        .collect();
    println!(
        "    AirportCode로 \"icn\" 찾기: {:?}, &str로 \"ICN\" 찾기: {:?}",
        strict.get(&AirportCode::new("icn")),
        strict.get("ICN")
    );
    // 출력:     AirportCode로 "icn" 찾기: Some("인천"), &str로 "ICN" 찾기: Some("인천")
    // str로 찾을 때는 str의 비교 규칙(대소문자 구분)을 따릅니다. 일관되므로 결과를 예측할 수 있습니다.
    let input = " gmp ";
    println!(
        "    &str로 {input:?} 찾기: {:?}, 정규화한 뒤 찾기: {:?}",
        strict.get(input),
        strict.get(input.trim().to_ascii_uppercase().as_str())
    );
    // 출력:     &str로 " gmp " 찾기: None, 정규화한 뒤 찾기: Some("김포")
    if !strict.contains_key("PUS") {
        strict.insert(AirportCode::new("pus"), "김해");
    }
    println!("    PUS를 새로 추가한 뒤 항목 수: {}", strict.len());
    // 출력:     PUS를 새로 추가한 뒤 항목 수: 4

    println!("3. 다른 선택: Borrow 없이 키 타입으로만 찾게 합니다");
    let only_keys: HashMap<StrictCode, &str> = entries
        .iter()
        .map(|&(code, name)| (StrictCode::new(code), name))
        .collect();
    // only_keys.get("ICN")은 컴파일되지 않습니다(compile_fail/e0308_hashmap_get_without_borrow.rs).
    // 찾을 때마다 키를 만들어야 하지만(할당 1번), 잘못 찾을 길이 없습니다.
    let wanted = StrictCode::new("cju");
    println!("    {} 찾기: {:?}", wanted.as_ref(), only_keys.get(&wanted));
    // 출력:     CJU 찾기: Some("제주")
}
