// let...else는 패턴이 맞으면 바인딩을 바깥 스코프에 남기고, 맞지 않으면 else 블록을 실행합니다.
// else 블록은 반드시 return, continue, break, panic! 등으로 흐름을 끊어야(diverge) 합니다.
// 덕분에 "값이 제대로 있는 정상 흐름", 즉 행복한 경로(happy path)를 들여쓰기 없이 위에서 아래로 쓸 수 있습니다.
//
// 실행: cargo run -p p0-ch6-enums-pattern-matching --example 07_let_else

// 설정 파일의 한 줄(예: "port=8080")을 읽어 포트를 설정합니다. 세 가지를 차례로 확인해야 합니다.
//   1) '='가 있는가?   split_once('=')는 처음 나오는 '=' 앞뒤를 (앞, 뒤) 튜플로 나눠 Some에 담고, 없으면 None을 돌려줍니다.
//   2) 키가 port인가?
//   3) 값이 u16 숫자인가? parse::<u16>()은 "u16으로 바꿔라"라고 타입을 직접 알려 주는 문법입니다.
//      parse는 성공하면 Ok(값), 실패하면 Err(에러)를 돌려줍니다. ch2에서 match로 나눴던 바로 그 Result입니다.

// 버전 1: if let을 겹쳐 쓰면, 확인할 것이 늘수록 들여쓰기가 깊어지고
// 각 실패 메시지가 자기 조건에서 멀리 떨어진 아래쪽 else에 놓입니다.
fn apply_nested(line: &str) {
    if let Some((key, value)) = line.split_once('=') {
        if key.trim() == "port" {
            if let Ok(port) = value.trim().parse::<u16>() {
                println!("    포트를 {port}번으로 설정합니다");
            } else {
                println!("    [건너뜀] 포트 번호가 아닙니다: {value}");
            }
        } else {
            println!("    [건너뜀] 모르는 설정입니다: {key}");
        }
    } else {
        println!("    [건너뜀] '='가 없습니다: {line}");
    }
}

// 버전 2: let...else로 쓰면, 실패하는 경우를 그 자리에서 처리하고 바로 빠져나갑니다.
// 확인을 통과한 뒤의 코드는 들여쓰기 없이 이어지고, 꺼낸 key, value, port를 그대로 쓸 수 있습니다.
fn apply(line: &str) {
    let Some((key, value)) = line.split_once('=') else {
        println!("    [건너뜀] '='가 없습니다: {line}");
        return;
    };
    // if let과 달리 key, value는 이 아래 함수 끝까지 쓸 수 있습니다.

    if key.trim() != "port" {
        println!("    [건너뜀] 모르는 설정입니다: {key}");
        return;
    }

    let Ok(port) = value.trim().parse::<u16>() else {
        println!("    [건너뜀] 포트 번호가 아닙니다: {value}");
        return;
    };

    println!("    포트를 {port}번으로 설정합니다");
}

fn main() {
    let lines = [
        "port=8080",
        "port = 443",
        "host=localhost",
        "port=abc",
        "# 주석",
    ];

    println!("1. if let을 겹쳐 쓴 버전");
    for line in lines {
        apply_nested(line);
    }
    // 출력:
    //     포트를 8080번으로 설정합니다
    //     포트를 443번으로 설정합니다
    //     [건너뜀] 모르는 설정입니다: host
    //     [건너뜀] 포트 번호가 아닙니다: abc
    //     [건너뜀] '='가 없습니다: # 주석

    println!("2. let...else 버전 (결과는 같습니다)");
    for line in lines {
        apply(line);
    }
    // 출력:
    //     포트를 8080번으로 설정합니다
    //     포트를 443번으로 설정합니다
    //     [건너뜀] 모르는 설정입니다: host
    //     [건너뜀] 포트 번호가 아닙니다: abc
    //     [건너뜀] '='가 없습니다: # 주석

    println!("3. 반복문 안에서는 else에서 continue로 다음 차례로 넘어갈 수 있습니다");
    let inputs = ["10", "스물", "30", "70000", "5"];
    let mut total: u16 = 0;
    for text in inputs {
        // 70000은 u16의 범위(0~65535)를 넘으므로 parse가 실패합니다.
        let Ok(n) = text.parse::<u16>() else {
            println!("    건너뜀: {text}");
            continue;
        };
        total += n;
    }
    println!("    합계: {total}");
    // 출력:
    //     건너뜀: 스물
    //     건너뜀: 70000
    //     합계: 45
}
