// 기대하는 에러: error[E0277]: `?` couldn't convert the error to `SetError`
// 핵심 메시지: the trait `From<&str>` is not implemented for `SetError`
//             help: the trait `From<&str>` is not implemented for `SetError`
//                   but trait `From<ParseIntError>` is implemented for it
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0277_ok_or_without_from.rs --out-dir /tmp/rust-er-ch3
//
// ok_or로 Option을 Result로 바꾼 것까지는 맞지만, 에러 값으로 문자열을 넣었습니다.
// 그래서 ok_or의 결과는 Result<_, &str>이고, ?는 &str을 함수의 에러 타입인 SetError로 바꾸려고 From<&str>을 찾습니다.
// SetError에는 From<ParseIntError>만 있으므로 E0277입니다. 아래 parse()?가 문제없이 통과하는 것과 비교해 보세요.
// 고치는 방법: ok_or에 처음부터 함수의 에러 타입 값을 넣습니다.
//   let (reps, _) = line.split_once('x').ok_or(SetError::NoSeparator)?;
// (examples/03_error_mapping.rs의 volume_map_err, volume_from)

use std::num::ParseIntError;

#[derive(Debug)]
enum SetError {
    NoSeparator,
    BadReps(ParseIntError),
}

impl From<ParseIntError> for SetError {
    fn from(e: ParseIntError) -> Self {
        SetError::BadReps(e)
    }
}

fn reps_of(line: &str) -> Result<u32, SetError> {
    let (reps, _) = line.split_once('x').ok_or("구분자 x가 없습니다")?;
    let reps: u32 = reps.trim().parse()?;
    Ok(reps)
}

fn main() {
    println!("{:?}", reps_of("12 x 40.5"));
    println!("{:?}", reps_of("12 40.5"));
    if let Err(SetError::BadReps(e)) = reps_of("열두 x 40.5") {
        println!("횟수를 읽지 못했습니다: {e}");
    }
}
