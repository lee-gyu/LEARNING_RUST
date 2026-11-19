// 지침: 원소의 타입이 서로 같아 구별되지 않는 튜플 대신, 필드에 이름이 있는 구조체를 씁니다.
// 비권장: 좌석 번호를 (줄, 칸) 튜플로 돌려주고, 호출한 쪽이 (x, y) 순서로 읽습니다.
// 권장:   SeatPos { row, col } 구조체로 돌려주고, 호출한 쪽이 이름으로 읽습니다.
//
// 실행: cargo run -p er-ch1-express-data-structures --example 02_named_fields

// 작은 공연장의 좌석 배치입니다. 한 줄에 5석씩 4줄이고, 좌석 번호는 0부터 매깁니다.
const SEATS_PER_ROW: usize = 5;
const ROWS: usize = 4;

// ---- 비권장 ----------------------------------------------------------------

// 돌려주는 값은 (줄, 칸)입니다. 하지만 시그니처의 (usize, usize)는 그 순서를 말해 주지 않습니다.
fn seat_position_tuple(seat_no: usize) -> (usize, usize) {
    (seat_no / SEATS_PER_ROW, seat_no % SEATS_PER_ROW)
}

// ---- 권장 ------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct SeatPos {
    row: usize,
    col: usize,
}

fn seat_position(seat_no: usize) -> SeatPos {
    SeatPos {
        row: seat_no / SEATS_PER_ROW,
        col: seat_no % SEATS_PER_ROW,
    }
}

// 배치도를 그리는 쪽은 화면 좌표(x는 가로, y는 세로)로 생각합니다.
// 같은 정보를 다른 이름 체계로 다루는 두 코드가 만나는 곳에서 순서 실수가 생깁니다.
fn draw(x: usize, y: usize) {
    for row in 0..ROWS {
        let line: String = (0..SEATS_PER_ROW)
            .map(|col| if row == y && col == x { '#' } else { '.' })
            .collect();
        println!("    {line}");
    }
}

fn main() {
    let seat_no = 8; // 둘째 줄의 넷째 칸(0부터 세면 1줄 3칸)

    println!("[비권장] 튜플 (usize, usize)");
    // 호출한 쪽은 "위치니까 (x, y)겠지"라고 생각하고 풀어 씁니다.
    // 두 원소의 타입이 같으므로 순서를 바꿔 읽어도 컴파일러는 알 수 없습니다.
    let (x, y) = seat_position_tuple(seat_no);
    println!("  x={x}, y={y}");
    draw(x, y);
    // 출력:
    //   x=1, y=3
    //     .....
    //     .....
    //     .....
    //     .#...

    println!("[권장] 구조체 SeatPos {{ row, col }}");
    // 필드 이름으로 꺼내므로 순서를 기억할 필요가 없습니다.
    let pos = seat_position(seat_no);
    println!("  {pos:?}");
    draw(pos.col, pos.row);
    // 출력:
    //   SeatPos { row: 1, col: 3 }
    //     .....
    //     ...#.
    //     .....
    //     .....

    // 구조체 패턴으로 풀 때도 필드 이름으로 맞추므로, 적는 순서가 결과를 바꾸지 않습니다.
    let SeatPos { col, row } = pos;
    println!("  이름으로 풀기: col={col}, row={row}");
    // 출력:   이름으로 풀기: col=3, row=1
}
