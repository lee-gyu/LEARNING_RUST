// 지침: f64 필드가 있는 타입에는 PartialEq와 PartialOrd까지만 derive합니다(Eq, Ord, Hash는 불가).
//       NaN이 끼면 "자기 자신과 같다"가 깨지므로, 정렬에는 total_cmp를 쓰고
//       맵의 키나 집합의 원소가 필요하면 정수 표현으로 바꾼 타입을 따로 둡니다.
// 비권장: NaN이 든 값을 ==와 contains로 찾기, partial_cmp(..).unwrap_or(Equal)로 정렬
// 권장:   total_cmp로 정렬, 마이크로도(1e-6도) 정수로 바꾼 PinKey에 Eq/Ord/Hash를 derive
//
// 지도 앱이 주소를 좌표로 바꿔 핀을 찍습니다. 주소 변환에 실패한 핀은 위도가 NaN입니다.
//
// 실행: cargo run -p er-ch10-standard-traits --example 04_float_fields

use std::cmp::Ordering;
use std::collections::HashSet;

// f64 필드가 있으므로 Eq와 Hash는 derive할 수 없습니다(E0277,
// compile_fail/e0277_hash_with_f64_field.rs). PartialEq와 PartialOrd는 됩니다.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Pin {
    lat: f64,
    lng: f64,
}

const fn pin(lat: f64, lng: f64) -> Pin {
    Pin { lat, lng }
}

const PINS: [(&str, Pin); 5] = [
    ("서울시청", pin(37.5663, 126.9779)),
    ("주소 변환 실패", pin(f64::NAN, 127.0)),
    ("부산역", pin(35.1152, 129.0422)),
    ("강릉역", pin(37.7641, 128.8996)),
    ("대전역", pin(36.3326, 127.4342)),
];

/// 집합이나 맵의 키로 쓸 정수 표현입니다. 위도·경도를 백만 배 한 정수로 담습니다.
// 정수 필드뿐이므로 비교·해시 트레이트를 모두 derive할 수 있고, 필드 순서(위도 먼저)가
// 정렬 순서가 됩니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PinKey {
    lat_e6: i32,
    lng_e6: i32,
}

impl PinKey {
    /// NaN이나 범위를 벗어난 좌표는 키로 만들지 않습니다.
    fn from_pin(pin: Pin) -> Option<PinKey> {
        let valid = (-90.0..=90.0).contains(&pin.lat) && (-180.0..=180.0).contains(&pin.lng);
        // 범위 검사가 NaN도 걸러 냅니다(NaN과의 비교는 모두 false).
        // 그래서 아래 `as` 변환은 잘리거나 넘치지 않습니다.
        valid.then(|| PinKey {
            lat_e6: (pin.lat * 1e6).round() as i32,
            lng_e6: (pin.lng * 1e6).round() as i32,
        })
    }
}

fn names_by<F>(mut compare: F) -> Vec<&'static str>
where
    F: FnMut(&Pin, &Pin) -> Ordering,
{
    let mut pins = PINS.to_vec();
    pins.sort_by(|(_, a), (_, b)| compare(a, b));
    pins.iter().map(|(name, _)| *name).collect()
}

fn main() {
    let pins: Vec<Pin> = PINS.iter().map(|(_, p)| *p).collect();

    println!("[비권장] NaN이 든 값을 ==로 찾기");
    let failed = pins[1];
    let same = failed; // 비트까지 똑같은 사본
    println!("  failed == 그 사본? {}", failed == same);
    // 출력:   failed == 그 사본? false
    println!(
        "  목록에 들어 있는 failed를 contains로 찾기: {}",
        pins.contains(&failed)
    );
    // 출력:   목록에 들어 있는 failed를 contains로 찾기: false
    println!("  서울시청 찾기: {}", pins.contains(&pins[0]));
    // 출력:   서울시청 찾기: true

    println!("[비권장] partial_cmp(..).unwrap_or(Equal)로 위도 정렬(남→북)");
    // NaN과의 비교를 Equal로 덮으면 비교 함수가 전순서가 아니게 됩니다.
    // 문서는 이때 정렬 결과가 정해지지 않으며 패닉할 수도 있다고 합니다.
    // (이 입력은 이 툴체인에서 패닉하지 않고 아래 결과를 낸다는 것을 확인했습니다.)
    let broken = names_by(|a, b| a.lat.partial_cmp(&b.lat).unwrap_or(Ordering::Equal));
    println!("  {broken:?}");
    // 출력:   ["서울시청", "주소 변환 실패", "부산역", "대전역", "강릉역"]

    println!("[권장] total_cmp로 위도 정렬(남→북)");
    // total_cmp는 IEEE 754의 전순서를 따릅니다. 양의 NaN(f64::NAN)은 모든 수보다 뒤에 섭니다.
    let sorted = names_by(|a, b| a.lat.total_cmp(&b.lat));
    println!("  {sorted:?}");
    // 출력:   ["부산역", "대전역", "서울시청", "강릉역", "주소 변환 실패"]

    println!("[권장] 키가 필요하면 정수 표현으로");
    // 다른 출처에서 같은 좌표를 한 번 더 받았습니다.
    let mut incoming = pins.clone();
    incoming.push(pin(35.1152, 129.0422));
    let mut keys = HashSet::new();
    let mut rejected = 0;
    for pin in &incoming {
        match PinKey::from_pin(*pin) {
            Some(key) => {
                keys.insert(key);
            }
            None => rejected += 1,
        }
    }
    println!(
        "  핀 {}개 → 서로 다른 좌표 {}개, 거절 {}개",
        incoming.len(),
        keys.len(),
        rejected
    );
    // 출력:   핀 6개 → 서로 다른 좌표 4개, 거절 1개
    let southmost = keys.iter().min();
    println!("  가장 남쪽: {southmost:?}");
    // 출력:   가장 남쪽: Some(PinKey { lat_e6: 35115200, lng_e6: 129042200 })
}
