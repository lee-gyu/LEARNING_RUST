// 기대하는 에러: E0451
//   error[E0451]: field `bases` of struct `DnaSeq` is private
//
// 예제 04의 DnaSeq를 모듈 밖에서 구조체 리터럴로 직접 만들어, from_bytes의 검사를 건너뛰려 합니다.
// bases 필드가 비공개라서 막힙니다. 이것이 막히지 않는다면 0xFF가 든 DnaSeq가 생기고,
// 안전한 메서드 as_str()가 올바른 UTF-8이 아닌 &str을 만들어 미정의 동작이 됩니다.
// 만든 뒤에 고치는 길(seq.bases.push(0xFF))도 같은 이유로 E0616입니다.
// 안전한 코드만으로는 불변식을 깰 수 없다는 것을 컴파일러가 가시성 규칙으로 지켜 줍니다.
//
// 확인: rustc --edition 2024 compile_fail/e0451_private_field_bypass.rs --out-dir /tmp/er-ch16

mod dna {
    /// 불변식: bases에는 A, C, G, T 바이트만 있다.
    pub struct DnaSeq {
        bases: Vec<u8>,
    }

    impl DnaSeq {
        pub fn from_bytes(bytes: Vec<u8>) -> Option<DnaSeq> {
            let ok = bytes.iter().all(|b| matches!(b, b'A' | b'C' | b'G' | b'T'));
            ok.then_some(DnaSeq { bases: bytes })
        }

        pub fn as_str(&self) -> &str {
            // SAFETY: 불변식에 따라 bases는 ASCII이고, ASCII 바이트열은 올바른 UTF-8입니다.
            unsafe { std::str::from_utf8_unchecked(&self.bases) }
        }
    }
}

fn main() {
    let checked = dna::DnaSeq::from_bytes(b"GATTACA".to_vec());
    println!("{:?}", checked.as_ref().map(|s| s.as_str()));

    let bad = vec![b'A', 0xFF];
    let forged = dna::DnaSeq { bases: bad }; // E0451
    println!("{}", forged.as_str());
}
