// 지침 4: 메서드 시그니처로는 드러낼 수 없는 약속은 메서드가 없는 마커 트레이트로 표현하고,
// 그 약속이 필요한 함수는 마커 트레이트를 바운드로 요구합니다.
//
// 이 예제의 약속은 멱등(idempotent), 즉 "여러 번 적용해도 한 번 적용한 것과 결과가 같다"입니다.
// `apply(&self, &mut Account)`라는 시그니처는 두 명령이 똑같아서 이 차이를 담지 못합니다.
//
// 실행: cargo run -p er-ch2-express-common-behavior --example 04_marker_trait

#[derive(Debug)]
struct Account {
    limit: u32,
    points: u32,
}

trait Command {
    fn apply(&self, account: &mut Account);
}

/// 한도를 정해진 값으로 맞춥니다. 몇 번 적용해도 결과가 같습니다.
struct SetLimit(u32);
/// 포인트를 적립합니다. 적용할 때마다 늘어납니다.
struct AddPoints(u32);

impl Command for SetLimit {
    fn apply(&self, account: &mut Account) {
        account.limit = self.0;
    }
}

impl Command for AddPoints {
    fn apply(&self, account: &mut Account) {
        account.points += self.0;
    }
}

/// 마커 트레이트: 메서드가 없습니다. 구현하는 쪽이 "이 명령은 멱등이다"라고 약속하는 표시입니다.
/// 컴파일러는 약속이 사실인지 검사하지 못하므로, 구현은 그 타입을 만든 사람이 책임지고 붙입니다.
trait Idempotent: Command {}

impl Idempotent for SetLimit {}
// AddPoints에는 붙이지 않습니다. 두 번 적용하면 두 번 적립되기 때문입니다.

/// 응답이 사라지는 불안정한 연결을 흉내 냅니다.
/// 명령은 서버에 적용되지만, 처음 `lost_replies`번은 응답이 사라져 "실패"로 보입니다.
struct FlakyLink {
    lost_replies: u32,
}

impl FlakyLink {
    fn send(&mut self, command: &impl Command, account: &mut Account) -> bool {
        command.apply(account);
        if self.lost_replies > 0 {
            self.lost_replies -= 1;
            false
        } else {
            true
        }
    }
}

/// 비권장: 어떤 명령이든 성공할 때까지 다시 보냅니다. 돌려주는 값은 보낸 횟수입니다.
fn send_with_retry_any<C: Command>(
    link: &mut FlakyLink,
    command: &C,
    account: &mut Account,
) -> u32 {
    let mut attempts = 1;
    while !link.send(command, account) && attempts < 3 {
        attempts += 1;
    }
    attempts
}

/// 권장: 다시 보내도 안전한, 멱등이라고 약속된 명령만 받습니다. 본문은 비권장 판과 같습니다.
fn send_with_retry<C: Idempotent>(link: &mut FlakyLink, command: &C, account: &mut Account) -> u32 {
    let mut attempts = 1;
    while !link.send(command, account) && attempts < 3 {
        attempts += 1;
    }
    attempts
}

/// 멱등이 아닌 명령은 한 번만 보내고, 실패 여부를 부르는 쪽에 알립니다.
fn send_once(link: &mut FlakyLink, command: &impl Command, account: &mut Account) -> bool {
    link.send(command, account)
}

fn main() {
    println!("1. 비권장: 아무 명령이나 재시도");
    let mut account = Account {
        limit: 100,
        points: 0,
    };
    let mut link = FlakyLink { lost_replies: 1 };
    let tries = send_with_retry_any(&mut link, &AddPoints(500), &mut account);
    println!(
        "    500포인트 적립, {tries}번 보냄 → points = {}",
        account.points
    );
    // 출력:     500포인트 적립, 2번 보냄 → points = 1000
    // 첫 번째 전송도 실제로는 적용됐으므로 두 번 적립됐습니다. 컴파일은 되지만 틀린 결과입니다.

    println!("2. 권장: 멱등인 명령은 재시도해도 결과가 같습니다");
    let mut account = Account {
        limit: 100,
        points: 0,
    };
    let mut link = FlakyLink { lost_replies: 1 };
    let tries = send_with_retry(&mut link, &SetLimit(300), &mut account);
    println!(
        "    한도 300으로 설정, {tries}번 보냄 → limit = {}",
        account.limit
    );
    // 출력:     한도 300으로 설정, 2번 보냄 → limit = 300

    println!("3. 권장: 멱등이 아닌 명령은 재시도 함수에 넘길 수 없습니다");
    // send_with_retry(&mut link, &AddPoints(500), &mut account)는
    // E0277: the trait bound `AddPoints: Idempotent` is not satisfied 로 거부됩니다.
    // 그래서 부르는 쪽은 한 번만 보내고, 실패하면 서버에 결과를 물어보는 등 다른 방법을 고릅니다.
    let mut link = FlakyLink { lost_replies: 1 };
    let ok = send_once(&mut link, &AddPoints(500), &mut account);
    println!("    응답 받음: {ok}, 최종 상태 {account:?}");
    // 출력:     응답 받음: false, 최종 상태 Account { limit: 300, points: 500 }
}
