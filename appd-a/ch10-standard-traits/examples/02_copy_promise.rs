// 지침: Copy는 "비트를 복사한 사본이 원본과 똑같이 쓰여도 괜찮다"는 약속입니다.
//       값처럼 행동하는 작은 타입에만 붙이고, 한 번만 써야 하는 권한이나 제자리에서
//       고쳐 가며 쓰는 상태에는 붙이지 않습니다. 한 번 붙이면 떼기 어렵습니다.
// 비권장: 송금 승인 CopyApproval에 Copy (같은 승인으로 두 번 송금)
//         투표함의 집계 CopyTally에 Copy (꺼내서 고친 것이 사본이라 표가 사라짐)
// 권장:   Approval에는 Clone도 Copy도 없음, Tally에는 Copy 없음
//
// 실행: cargo run -p er-ch10-standard-traits --example 02_copy_promise

// ---- 한 번만 써야 하는 승인 --------------------------------------------------

#[derive(Debug)]
struct Account {
    balance: i64,
}

// 비권장: 필드가 정수 하나라 Copy를 붙일 수 있고, 컴파일러도 막지 않습니다.
#[derive(Debug, Clone, Copy)]
struct CopyApproval {
    amount: i64,
}

// 승인을 값으로 받아 소비합니다. 받는 쪽은 "한 승인 = 한 번 송금"을 기대했습니다.
fn transfer_copy(account: &mut Account, approval: CopyApproval) {
    account.balance -= approval.amount;
}

// 권장: Copy도 Clone도 없습니다. 승인을 복제할 길이 없으므로 넘기면 끝입니다.
#[derive(Debug)]
struct Approval {
    amount: i64,
}

fn transfer(account: &mut Account, approval: Approval) {
    account.balance -= approval.amount;
}

// ---- 제자리에서 고쳐 쓰는 상태 -----------------------------------------------

#[derive(Debug, Clone, Copy)]
struct CopyTally {
    yes: u32,
    no: u32,
}

#[derive(Debug)]
struct CopyBallotBox {
    tally: CopyTally,
}

// 비권장: tally가 Copy라서 첫 줄은 "꺼내기"가 아니라 "복사"입니다.
// 고친 것은 지역 변수인 사본이고, 투표함의 집계는 그대로입니다.
// 사본을 읽어 총 표 수를 돌려주므로 rustc의 unused_assignments 경고도 나지 않습니다.
fn vote_yes_copy(ballot_box: &mut CopyBallotBox) -> u32 {
    let mut tally = ballot_box.tally;
    tally.yes += 1;
    tally.yes + tally.no
}

#[derive(Debug, Clone)]
struct Tally {
    yes: u32,
    no: u32,
}

#[derive(Debug)]
struct BallotBox {
    tally: Tally,
}

// 권장: Tally가 Copy가 아니면 `let mut tally = ballot_box.tally;`는
// "빌린 값에서 옮길 수 없다"는 E0507이 되어, 사본을 고치는 실수가 컴파일 단계에서 드러납니다.
// 그래서 처음부터 가변 참조로 고칩니다.
fn vote_yes(ballot_box: &mut BallotBox) -> u32 {
    let tally = &mut ballot_box.tally;
    tally.yes += 1;
    tally.yes + tally.no
}

fn main() {
    println!("[비권장] Copy인 송금 승인");
    let mut account = Account { balance: 100_000 };
    let approval = CopyApproval { amount: 30_000 };
    transfer_copy(&mut account, approval);
    transfer_copy(&mut account, approval); // 실수로 한 번 더. Copy라서 컴파일됩니다.
    println!(
        "  승인 {}원 1건, 잔액 {}원",
        approval.amount, account.balance
    );
    // 출력:   승인 30000원 1건, 잔액 40000원

    println!("[권장] Copy도 Clone도 없는 송금 승인");
    let mut account = Account { balance: 100_000 };
    let approval = Approval { amount: 30_000 };
    let amount = approval.amount;
    transfer(&mut account, approval);
    // transfer(&mut account, approval); // E0382: 이미 옮겨진 승인을 다시 쓸 수 없습니다.
    println!("  승인 {amount}원 1건, 잔액 {}원", account.balance);
    // 출력:   승인 30000원 1건, 잔액 70000원

    println!("[비권장] Copy인 집계");
    let mut copy_box = CopyBallotBox {
        tally: CopyTally { yes: 0, no: 0 },
    };
    let first = vote_yes_copy(&mut copy_box);
    let second = vote_yes_copy(&mut copy_box);
    println!("  찬성 투표 뒤 총 {first}표, 한 번 더 투표 뒤 총 {second}표");
    // 출력:   찬성 투표 뒤 총 1표, 한 번 더 투표 뒤 총 1표
    println!(
        "  찬성 2표를 넣었는데 찬성 {}, 반대 {}",
        copy_box.tally.yes, copy_box.tally.no
    );
    // 출력:   찬성 2표를 넣었는데 찬성 0, 반대 0

    println!("[권장] Copy가 아닌 집계");
    let mut ballot_box = BallotBox {
        tally: Tally { yes: 0, no: 0 },
    };
    let first = vote_yes(&mut ballot_box);
    let second = vote_yes(&mut ballot_box);
    println!("  찬성 투표 뒤 총 {first}표, 한 번 더 투표 뒤 총 {second}표");
    // 출력:   찬성 투표 뒤 총 1표, 한 번 더 투표 뒤 총 2표
    // 사본이 필요하면 clone()으로 드러내 놓고 만듭니다.
    let snapshot = ballot_box.tally.clone();
    println!(
        "  찬성 2표를 넣었고 찬성 {}, 반대 {}",
        snapshot.yes, snapshot.no
    );
    // 출력:   찬성 2표를 넣었고 찬성 2, 반대 0
}
