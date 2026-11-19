// 기대하는 결과: 2015·2018·2021·2024 모든 에디션에서 컴파일 에러
// 핵심 메시지: error: `crate` cannot be a raw identifier
//              error: `self` cannot be a raw identifier
//
// raw 식별자 r#는 대부분의 키워드를 이름으로 쓰게 해 주지만, 경로를 이루는 키워드인
// crate, self, super, Self와 밑줄 _는 r#를 붙여도 이름이 될 수 없습니다.
// 이 이름들은 경로의 시작점(crate::, self::, super::, Self::)이라서 이름으로 허용하면 경로가 무엇을
// 가리키는지 모호해지기 때문입니다. 또 2015 에디션에서도 이미 이름으로 쓸 수 없었으므로, 옛 크레이트가
// 이 이름으로 무언가를 공개했을 리도 없습니다. raw 식별자가 필요한 상황 자체가 생기지 않습니다.
// 고치는 방법: krate, self_ 같은 다른 이름을 씁니다.
//
// 확인(실패): rustc --edition 2024 compile_fail/raw_ident_reserved.rs --out-dir /tmp/rust-p2-ch2
// 확인(실패): rustc --edition 2015 compile_fail/raw_ident_reserved.rs --out-dir /tmp/rust-p2-ch2

struct Package {
    r#crate: String, // → 에러: `crate` cannot be a raw identifier
    r#type: String,  // type은 r#를 붙이면 됩니다
}

fn main() {
    let r#self = Package {
        // → 에러: `self` cannot be a raw identifier
        r#crate: String::from("scorekit"),
        r#type: String::from("lib"),
    };
    println!("{} {}", r#self.r#crate, r#self.r#type);
}
