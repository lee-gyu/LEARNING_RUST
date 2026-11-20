//! tinytlv의 주된 사용법: 레코드를 만들어 기록하고, 받은 바이트를 다시 읽어 해석합니다.
//!
//! 실행: cargo run --example sensor_report
//!
//! 예제 프로그램은 사용자가 옮겨 쓸 코드이므로 `unwrap()` 대신 `?`를 씁니다.

use std::error::Error;

use tinytlv::{Record, encode, parse};

const TEMPERATURE: u8 = 1; // 값: 0.1도 단위 i16, 빅 엔디언
const HUMIDITY: u8 = 2; // 값: 퍼센트 1바이트
const DOOR_CLOSED: u8 = 3; // 값 없음: 레코드가 있다는 것 자체가 정보
const DEVICE_NAME: u8 = 4; // 값: UTF-8 문자열

fn main() -> Result<(), Box<dyn Error>> {
    let readings = [
        Record::new(DEVICE_NAME, "3층 서버실 북쪽 벽 온습도 센서")?,
        Record::new(TEMPERATURE, 215_i16.to_be_bytes())?,
        Record::new(HUMIDITY, [48])?,
        Record::new(DOOR_CLOSED, [])?,
    ];
    let bytes = encode(&readings);
    println!("보낼 바이트 {}개", bytes.len());

    for record in parse(&bytes)? {
        match record.tag() {
            DEVICE_NAME => println!("장치: {}", str::from_utf8(record.value())?),
            TEMPERATURE => {
                let raw = i16::from_be_bytes(record.value().try_into()?);
                println!("온도 {:.1}도", f32::from(raw) / 10.0);
            }
            HUMIDITY => {
                let [percent]: [u8; 1] = record.value().try_into()?;
                println!("습도 {percent}%");
            }
            DOOR_CLOSED => println!("문 닫힘"),
            other => println!("모르는 태그 {other}"),
        }
    }
    Ok(())
}
