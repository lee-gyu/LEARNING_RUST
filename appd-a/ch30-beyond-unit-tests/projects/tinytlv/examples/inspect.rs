//! 받은 바이트를 레코드 단위로 풀어 보여 줍니다. 바이트는 16진수 인자로 받습니다.
//!
//! 실행: cargo run --example inspect -- 01 02 00 d7 03 00
//!       인자가 없으면 안에 든 예시 바이트를 씁니다.

use std::env;
use std::error::Error;

const SAMPLE: [u8; 9] = [0x01, 0x02, 0x00, 0xd7, 0x02, 0x01, 0x30, 0x03, 0x00];

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let bytes = if args.is_empty() {
        SAMPLE.to_vec()
    } else {
        args.iter()
            .map(|a| u8::from_str_radix(a, 16))
            .collect::<Result<Vec<u8>, _>>()?
    };

    let records = tinytlv::parse(&bytes)?;
    println!("레코드 {}개", records.len());
    for (i, r) in records.iter().enumerate() {
        println!(
            "  #{i} 태그 {:3}  길이 {:3}  값 {:02x?}",
            r.tag(),
            r.value().len(),
            r.value()
        );
    }
    Ok(())
}
