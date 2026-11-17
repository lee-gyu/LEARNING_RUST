// 이 예제가 보여 주는 것: 참조 필드를 가진 구조체의 라이프타임, 라이프타임 생략 규칙 3가지,
//                        메서드에서의 라이프타임, 'static 라이프타임
// 실행: cargo run -p p0-ch10-generics-traits-lifetimes --example 08_struct_elision_static

// 필드에 참조를 담는 구조체는 라이프타임 매개변수를 선언하고 필드에 붙입니다.
// "ImportantExcerpt 인스턴스는 part가 가리키는 값보다 오래 살 수 없다"는 뜻입니다.
// ch5에서 E0106으로 막혔던 User 구조체도 같은 방법으로 쓸 수 있습니다(아래 User<'a>).
struct ImportantExcerpt<'a> {
    part: &'a str,
}

struct User<'a> {
    username: &'a str,
    email: &'a str,
}

// impl 뒤의 <'a>는 impl<T>의 <T>처럼 "이 블록의 'a는 매개변수다"라는 선언입니다.
// 라이프타임은 구조체 타입의 일부이므로 ImportantExcerpt<'a>까지 적어야 합니다.
impl<'a> ImportantExcerpt<'a> {
    // 반환값이 참조가 아니므로 라이프타임을 신경 쓸 필요가 없습니다.
    fn level(&self) -> i32 {
        3
    }

    // 입력 참조가 &self와 announcement 두 개인데도 표기 없이 컴파일됩니다.
    // 규칙 3: 메서드에 &self가 있으면 반환 참조에는 self의 라이프타임이 붙습니다.
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("    알림: {announcement}");
        self.part
    }
}

// 규칙 1 + 규칙 2가 적용되는 함수입니다. 입력 참조가 하나뿐이라 반환 참조는 그 입력에서 빌린 것으로 정해집니다.
// ch4에서 만든 first_word가 표기 없이 컴파일된 이유가 이것입니다.
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

// 위 함수에서 컴파일러가 채워 넣는 라이프타임을 직접 적은 모습입니다. 두 함수는 완전히 같습니다.
// Clippy는 생략할 수 있는 라이프타임을 적으면 needless_lifetimes 경고를 냅니다.
// 여기서는 생략 전후를 비교하려고 일부러 적었으므로 expect로 표시합니다.
#[expect(
    clippy::needless_lifetimes,
    reason = "생략 규칙이 채워 넣는 모습을 보여 주려고 일부러 적음"
)]
fn first_word_explicit<'a>(s: &'a str) -> &'a str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

// 규칙 2는 출력 참조가 여러 개여도 모두에 같은 라이프타임을 붙입니다. (&'a str, &'a str)과 같습니다.
fn split_at_space(s: &str) -> (&str, &str) {
    match s.find(' ') {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    }
}

// 입력 참조가 하나도 없으면 생략 규칙으로 반환 참조의 라이프타임을 정할 수 없습니다(E0106).
// 문자열 리터럴은 프로그램 바이너리 안에 있어 프로그램이 끝날 때까지 유효하므로 'static을 붙입니다.
fn level_name(level: i32) -> &'static str {
    match level {
        1 => "참고",
        2 => "중요",
        _ => "매우 중요",
    }
}

fn main() {
    println!("1. 참조 필드를 가진 구조체");
    let novel = String::from("나를 이스마엘이라 불러 다오. 몇 년 전의 일이다...");
    // find('.')는 첫 '.'의 바이트 위치를 Option으로 돌려줍니다. '.'은 1바이트 문자라 그 위치에서 잘라도 안전합니다(ch4).
    let end = novel.find('.').unwrap_or(novel.len());
    let excerpt = ImportantExcerpt {
        part: &novel[..end],
    };
    println!("    발췌: {}", excerpt.part);
    let user = User {
        username: "ferris",
        email: "ferris@example.com",
    };
    println!("    {} <{}>", user.username, user.email);
    // 출력:
    //     발췌: 나를 이스마엘이라 불러 다오
    //     ferris <ferris@example.com>
    // excerpt는 novel의 일부를 빌리고 있으므로, novel보다 먼저 drop되는 동안만 쓸 수 있습니다.
    // novel을 안쪽 블록에 두고 excerpt를 블록 밖에서 쓰면 E0597(`novel` does not live long enough)입니다.

    println!("2. 생략 규칙 1·2: 입력 참조가 하나면 표기를 생략할 수 있습니다");
    let sentence = "hello world";
    println!(
        "    first_word = {}, first_word_explicit = {}",
        first_word(sentence),
        first_word_explicit(sentence)
    );
    let (head, tail) = split_at_space("안녕 러스트");
    println!("    split_at_space → ({head}, {tail})");
    // 출력:
    //     first_word = hello, first_word_explicit = hello
    //     split_at_space → (안녕, 러스트)

    println!("3. 생략 규칙 3: 메서드의 반환 참조에는 self의 라이프타임이 붙습니다");
    println!("    level = {}", excerpt.level());
    let part = excerpt.announce_and_return_part("발췌를 소개합니다");
    println!("    돌려받은 부분: {part}");
    // 출력:
    //     level = 3
    //     알림: 발췌를 소개합니다
    //     돌려받은 부분: 나를 이스마엘이라 불러 다오

    println!("4. 'static: 프로그램이 끝날 때까지 유효한 참조");
    let label: &'static str = level_name(excerpt.level());
    let literal: &'static str = "문자열 리터럴은 모두 'static입니다";
    println!("    {label} / {literal}");
    // 출력: 매우 중요 / 문자열 리터럴은 모두 'static입니다
}
