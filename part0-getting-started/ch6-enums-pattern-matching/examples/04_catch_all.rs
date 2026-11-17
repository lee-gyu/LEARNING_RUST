// match는 가능한 모든 경우를 다뤄야 컴파일됩니다. 이것을 완전성 검사(exhaustiveness checking)라고 합니다.
// 일일이 적을 수 없는 나머지 경우는 이름 있는 포괄 패턴(other)이나 _ 패턴으로 한꺼번에 받습니다.
// 마지막 구역은 열거형에서 _를 쓸 때와 배리언트를 모두 적을 때의 차이를 보여 줍니다.
//
// 실행: cargo run -p p0-ch6-enums-pattern-matching --example 04_catch_all

// 보드게임에서 주사위를 굴린 결과에 따라 말을 움직입니다.
// u8은 0~255 중 어떤 값이든 될 수 있으므로, 1과 6만 적으면 나머지 254가지가 빠져 컴파일되지 않습니다(E0004).
fn move_piece(roll: u8) {
    match roll {
        1 => println!("    {roll}: 출발점으로 돌아갑니다"),
        6 => println!("    {roll}: 6칸 가고 한 번 더 던집니다"),
        // 이름 있는 포괄 패턴: 변수 이름 하나로 된 패턴은 어떤 값과도 맞고, 그 값을 그 이름에 묶습니다.
        // 위에서부터 차례로 맞춰 보므로 1과 6은 이 갈래까지 오지 않습니다.
        other => println!("    {other}: {other}칸 앞으로 갑니다"),
    }
}

// 나머지 경우의 값이 필요 없으면 _를 씁니다. _도 어떤 값과도 맞지만, 값을 어떤 이름에도 묶지 않습니다.
fn cheer(roll: u8) -> String {
    match roll {
        1 => String::from("아쉬워요!"),
        6 => String::from("최고예요!"),
        _ => String::from("좋아요."),
    }
}

// 나머지 경우에 아무것도 하지 않으려면 갈래의 값으로 유닛 ()을 씁니다.
// "다른 값은 일부러 무시한다"는 뜻이 코드에 드러나고, match의 완전성도 지켜집니다.
fn draw_card(roll: u8) {
    match roll {
        1 => println!("    {roll}: 벌칙 카드를 뽑습니다"),
        6 => println!("    {roll}: 보너스 카드를 뽑습니다"),
        _ => (),
    }
}

#[derive(Debug)]
enum Ticket {
    Adult,
    Teen,
    Child,
    Senior,
}

// _로 "나머지 전부"를 한 갈래에 몰아넣은 버전입니다.
// 나중에 Ticket에 새 배리언트(예: 영유아)를 추가해도 컴파일러는 아무 말도 하지 않고,
// 새 배리언트는 조용히 _ 갈래의 가격을 받게 됩니다.
fn price_with_wildcard(ticket: &Ticket) -> u32 {
    match ticket {
        Ticket::Adult => 15000,
        _ => 10000,
    }
}

// 배리언트를 모두 적은 버전입니다.
// 새 배리언트를 추가하면 이 match가 E0004로 컴파일되지 않으므로, 가격을 정하는 것을 잊을 수 없습니다.
// (compile_fail/e0004_missing_match_arm.rs가 바로 그 상황입니다.)
fn price_exhaustive(ticket: &Ticket) -> u32 {
    match ticket {
        Ticket::Adult => 15000,
        Ticket::Teen => 12000,
        Ticket::Child => 8000,
        Ticket::Senior => 10000,
    }
}

fn main() {
    println!("1. 이름 있는 포괄 패턴(other)은 나머지 값을 받아 이름에 묶습니다");
    move_piece(1);
    move_piece(4);
    move_piece(6);
    // 출력:
    //     1: 출발점으로 돌아갑니다
    //     4: 4칸 앞으로 갑니다
    //     6: 6칸 가고 한 번 더 던집니다

    println!("2. _ 패턴은 나머지 값을 받지만 묶지는 않습니다");
    println!("    1 → {}, 3 → {}, 6 → {}", cheer(1), cheer(3), cheer(6));
    // 출력:
    //     1 → 아쉬워요!, 3 → 좋아요., 6 → 최고예요!

    println!("3. _ => ()는 \"나머지는 아무것도 하지 않는다\"는 뜻입니다");
    draw_card(1);
    draw_card(3); // 아무것도 출력하지 않습니다.
    draw_card(6);
    // 출력:
    //     1: 벌칙 카드를 뽑습니다
    //     6: 보너스 카드를 뽑습니다

    println!("4. 열거형에서 _를 쓸지, 배리언트를 모두 적을지");
    let tickets = [Ticket::Adult, Ticket::Teen, Ticket::Child, Ticket::Senior];
    for ticket in &tickets {
        println!(
            "    {ticket:?}: _ 사용 {}원, 모두 적음 {}원",
            price_with_wildcard(ticket),
            price_exhaustive(ticket)
        );
    }
    // 출력:
    //     Adult: _ 사용 15000원, 모두 적음 15000원
    //     Teen: _ 사용 10000원, 모두 적음 12000원
    //     Child: _ 사용 10000원, 모두 적음 8000원
    //     Senior: _ 사용 10000원, 모두 적음 10000원
    // 두 함수의 결과가 다른 것은 _ 갈래가 Teen과 Child를 구분하지 않고 같은 값으로 처리했기 때문입니다.
}
