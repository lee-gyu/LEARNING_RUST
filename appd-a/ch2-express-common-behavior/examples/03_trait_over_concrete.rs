// 지침 3: 함수가 매개변수에게 바라는 것이 "어떤 동작"뿐이라면, 구체 타입 대신 그 동작을 묶은
// 트레이트로 받고 요구사항은 트레이트 바운드로 적습니다.
//
// 구체 타입으로 받으면 부르는 쪽은 그 타입 말고는 넘길 것이 없습니다. 이 예제에서는 "실제로
// 쓰지 않고 미리 보기"를 하고 싶은데, 디스크 타입만 받는 함수로는 미리 보기도 디스크에 써 버립니다.
//
// 실행: cargo run -p er-ch2-express-common-behavior --example 03_trait_over_concrete

/// 진짜 디스크를 흉내 낸 타입입니다. 쓸 때마다 "[디스크에 씀]" 줄을 출력해서
/// 되돌릴 수 없는 부수 효과가 일어났다는 것을 눈으로 보이게 합니다.
struct Disk {
    files: usize,
}

impl Disk {
    fn write_file(&mut self, name: &str, body: &str) -> usize {
        println!("    [디스크에 씀] {name}");
        self.files += 1;
        body.len()
    }
}

/// 비권장: 구체 타입 `Disk`만 받습니다.
fn backup_to_disk(disk: &mut Disk, notes: &[(&str, &str)]) -> usize {
    let mut bytes = 0;
    for (name, body) in notes {
        bytes += disk.write_file(name, body);
    }
    bytes
}

/// 백업에 필요한 동작은 "이름과 내용을 받아 저장하고 쓴 바이트 수를 알려 준다" 하나뿐입니다.
trait Storage {
    fn save(&mut self, name: &str, body: &str) -> usize;
}

impl Storage for Disk {
    fn save(&mut self, name: &str, body: &str) -> usize {
        self.write_file(name, body)
    }
}

/// 아무것도 쓰지 않고 무엇이 쓰일지만 기록하는 미리 보기 저장소.
/// `backup`을 만든 사람은 이 타입을 몰라도 됩니다. 부르는 쪽이 나중에 추가한 구현입니다.
struct Preview {
    planned: Vec<String>,
}

impl Storage for Preview {
    fn save(&mut self, name: &str, body: &str) -> usize {
        self.planned.push(name.to_string());
        body.len()
    }
}

/// 권장: `Storage`를 구현한 어떤 타입이든 받습니다. 본문은 비권장 판과 같습니다.
/// 바운드에는 이 함수가 실제로 쓰는 동작(`save`)만 요구합니다.
fn backup(storage: &mut impl Storage, notes: &[(&str, &str)]) -> usize {
    let mut bytes = 0;
    for (name, body) in notes {
        bytes += storage.save(name, body);
    }
    bytes
}

fn main() {
    let notes = [("todo.txt", "우유 사기"), ("idea.txt", "rust study")];

    println!("1. 비권장: 미리 보기를 하려 해도 Disk밖에 넘길 것이 없습니다");
    let mut disk = Disk { files: 0 };
    let bytes = backup_to_disk(&mut disk, &notes);
    println!(
        "    예상 크기 {bytes}바이트, 디스크의 파일 수 {}",
        disk.files
    );
    // 출력:     [디스크에 씀] todo.txt
    // 출력:     [디스크에 씀] idea.txt
    // 출력:     예상 크기 23바이트, 디스크의 파일 수 2
    // 크기를 알아냈지만, 그 대가로 파일이 이미 써졌습니다.

    println!("2. 권장: 미리 보기 저장소를 넘기면 디스크는 건드리지 않습니다");
    let mut disk = Disk { files: 0 };
    let mut preview = Preview {
        planned: Vec::new(),
    };
    let bytes = backup(&mut preview, &notes);
    println!(
        "    예상 크기 {bytes}바이트, 쓸 파일 {:?}, 디스크의 파일 수 {}",
        preview.planned, disk.files
    );
    // 출력:     예상 크기 23바이트, 쓸 파일 ["todo.txt", "idea.txt"], 디스크의 파일 수 0

    println!("3. 확인한 뒤에 같은 함수로 진짜 디스크에 씁니다");
    let bytes = backup(&mut disk, &notes);
    println!("    쓴 크기 {bytes}바이트, 디스크의 파일 수 {}", disk.files);
    // 출력:     [디스크에 씀] todo.txt
    // 출력:     [디스크에 씀] idea.txt
    // 출력:     쓴 크기 23바이트, 디스크의 파일 수 2
}
