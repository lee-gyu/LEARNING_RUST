// 기대하는 에러: 어느 쪽으로 빌드해도 실패합니다.
//   기능이 꺼진 빌드: E0560 struct `Message` has no field named `attachments`  (reports 모듈)
//   기능이 켜진 빌드: E0063 missing field `attachments` in initializer of `Message`  (alerts 모듈)
//
// 확인 방법 (<임시 디렉터리>는 산출물을 둘 아무 디렉터리):
//   rustc --edition 2024 --out-dir <임시 디렉터리> compile_fail/e0560_gated_pub_field.rs
//     → E0560으로 실패합니다.
//   rustc --edition 2024 --out-dir <임시 디렉터리> --cfg 'feature="attachments"' compile_fail/e0560_gated_pub_field.rs
//     → E0063으로 실패합니다.
//
// 한 파일로 줄이려고 라이브러리(notice)와 그것을 쓰는 두 크레이트(alerts, reports)를 한 크레이트의
// 모듈로 흉내 냈습니다. 실제로는 alerts와 reports가 notice의 기능이 켜졌는지 `cfg`로 물어볼 수
// 없습니다(`cfg(feature = ..)`는 자기 크레이트의 기능만 가리킴). 그러니 공개 필드를 기능으로
// 가리면, 구조체 리터럴로 그 타입을 만드는 쪽은 필드를 적어도 안 적어도 어느 한 빌드에서
// 깨집니다. 여러 크레이트로 재현한 것은 projects/gated_api/broken에 있습니다.

mod notice {
    pub struct Message {
        pub to: String,
        pub body: String,
        #[cfg(feature = "attachments")]
        pub attachments: Vec<String>,
    }
}

// 기능을 모르는 쪽: 필드를 적지 않았습니다. 기능이 켜지면 E0063.
mod alerts {
    use crate::notice::Message;

    pub fn disk_alert() -> Message {
        Message {
            to: "운영팀".to_string(),
            body: "디스크 사용량 93%".to_string(),
        }
    }
}

// 기능이 켜졌으리라 믿는 쪽: 필드를 적었습니다. 기능이 꺼지면 E0560.
mod reports {
    use crate::notice::Message;

    pub fn weekly() -> Message {
        Message {
            to: "팀장".to_string(),
            body: "주간 보고".to_string(),
            attachments: vec!["보고서.pdf".to_string()],
        }
    }
}

fn main() {
    for m in [alerts::disk_alert(), reports::weekly()] {
        println!("{} ← {}", m.to, m.body);
    }
}
