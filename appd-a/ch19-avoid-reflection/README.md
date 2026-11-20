# ch19. 리플렉션 피하기

Java, Go, Python 같은 언어에는 **리플렉션(reflection)**, 즉 프로그램이 실행 중에 자기 타입 정보를 들여다보는 기능이 있습니다. 값의 클래스 이름을 문자열로 얻고, 필드 목록을 훑고, "이 객체가 저 인터페이스도 구현했나"를 실행 중에 묻습니다. C++에는 이보다 좁은 **실행 중 타입 정보(RTTI)** 가 있어 `typeid`와 `dynamic_cast`로 실제 타입을 확인합니다. Rust에는 둘 다 없습니다. 표준 라이브러리의 `std::any`가 비슷한 기능을 조금 주지만, 책은 다른 방법이 없을 때만 쓰라고 권합니다. 이 챕터는 **다른 언어에서 리플렉션으로 풀던 문제를 Rust에서는 무엇으로 푸는가**를 다룹니다.

`std::any`의 API(`type_name`, `TypeId`, `Any`, 다운캐스트, 타입 맵)는 [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 2.6~2.8절에서 배웠으므로 여기서는 고르는 기준만 봅니다. 앞 부록 챕터가 이 챕터로 넘긴 내용도 다룹니다. [ch5](../ch5-type-conversions/README.md)와 [ch12](../ch12-generics-vs-trait-objects/README.md)가 넘긴 트레이트 객체 업캐스트의 쓰임, ch12 2.5절이 넘긴 "트레이트 객체에서 그 타입에만 있는 정보를 보려면 트레이트에 메서드를 더하거나 다운캐스트해야 한다"는 문제입니다. 두 방법 중 무엇을 고를지가 2.3절의 주제입니다.

- 원제: [Item 19: Avoid reflection](https://effective-rust.com/reflection.html) (David Drysdale, *Effective Rust*)
- 패키지: `er-ch19-avoid-reflection`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션)

다음 본편 챕터의 내용은 안다고 보고 다시 설명하지 않습니다.

- [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md) 2.6~2.8절: `type_name`, `TypeId`, `dyn Any`와 다운캐스트, 타입 맵, "상자 자체도 `Any`" 함정
- [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md) 2.4~2.10절: 트레이트 객체, vtable, 열거형과 트레이트 객체의 비교
- [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.11절(슈퍼트레이트), 2.20절(절차적 매크로와 derive)
- [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md) 2.5절: `Box<dyn Error>`의 다운캐스트

`type_name`이 돌려주는 문자열은 형식이 보장되지 않는 진단용입니다. 이 챕터에 옮긴 출력은 rustc 1.100.0의 결과이며 툴체인이 바뀌면 달라질 수 있습니다. `TypeId`는 값을 출력하지 않고 같은지만 비교합니다.

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Rust에 리플렉션이 없는 이유와 `std::any`가 답할 수 있는 질문의 범위를 설명하고, `type_name`의 결과로 분기하면 참조·제네릭 인자·모듈 경로·별칭 때문에 틀린다는 것을 예로 들어 설명할 수 있다.
2. 트레이트 객체에 `downcast_ref`를 줄줄이 시도하는 분기를 필수 메서드와 기본 구현이 있는 선택적 메서드(닫힌 집합이면 열거형)로 바꿔 쓸 수 있고, 새 타입을 더했을 때 두 판이 어떻게 다르게 반응하는지 설명할 수 있다.
3. `Any`로는 "이 값이 다른 트레이트도 구현했는가"를 물을 수 없음을 설명하고, 그 질문을 기능 조회 메서드, 트레이트 바운드, 마커 트레이트로 표현할 수 있다.
4. 트레이트 객체 업캐스트의 현재 동작(슈퍼트레이트 쪽으로만, 한 방향으로만)을 설명하고, `as_any` 우회 메서드 없이 `Any`를 슈퍼트레이트로 두어 다운캐스트하는 코드를 작성할 수 있다.
5. 필드 이름 문자열로 값을 꺼내는 코드 대신 derive가 컴파일할 때 만든 코드나 접근 함수를 쓰는 이유를 설명할 수 있다.

## 2. 핵심 개념

2.1절에서 Rust에 리플렉션이 없는 이유와 대신 쓰는 도구를 표로 본 뒤, 2.2~2.6절에서 지침마다 문제 상황과 두 형태를 비교합니다. 이 Item의 비권장 코드는 모두 **다른 언어의 리플렉션 습관을 `type_name`·`Any`로 옮긴 코드**이고, 컴파일은 되지만 실행 중에 조용히 틀린 결과를 냅니다. 예제는 모두 온라인 강의 플랫폼의 학습 자료(강의 영상, 퀴즈, 읽기 자료)와 활동 기록을 다룹니다.

### 2.1 지도: 리플렉션이 답하던 질문을 컴파일할 때 답한다

Rust의 타입은 컴파일할 때 모두 정해지고, 프로그램이 실행 중에 읽을 수 있는 타입 정보는 거의 남지 않습니다. 제네릭 함수는 쓰인 타입마다 따로 컴파일되고(단형성화), 구조체의 필드 이름이나 "이 타입이 구현한 트레이트 목록" 같은 정보는 실행 중에 조회할 수 있는 형태로 바이너리에 실리지 않습니다. 트레이트 객체가 실행 중에 들고 다니는 것도 데이터 포인터와 vtable(메서드 포인터 표)뿐이고, vtable에는 그 트레이트 객체를 다루는 데 필요한 정보(메서드 주소 등)만 있습니다. 그런 정보를 싣는 비용(크기와 속도)을 치르지 않는 대신, 실행 중에 물어볼 곳도 없습니다.

그래서 Rust에서는 리플렉션이 실행 중에 답하던 질문을 **컴파일할 때 답하도록** 바꿉니다. 실행 중에 공유 라이브러리로 불러오는 플러그인처럼 컴파일할 때 소스가 없는 코드도, 책은 리플렉션이 아니라 트레이트 객체로 다룬다고 설명합니다(ch12 2.2절).

| 하고 싶은 일 | 리플렉션 습관을 옮긴 비권장 코드 | 권장 | 절 |
| --- | --- | --- | --- |
| 타입에 따라 다르게 처리 | `type_name` 문자열 비교 | 트레이트 메서드 | 2.2 |
| 트레이트 객체에서 그 타입만의 정보 쓰기 | `downcast_ref` 사슬 | 필수·선택 메서드, 닫힌 집합이면 열거형 | 2.3 |
| "이 기능도 지원하나?" 묻기 | 아는 구체 타입만 다운캐스트 | 기능 조회 메서드, 바운드, 마커 트레이트 | 2.4 |
| 정말 구체 타입을 꺼내야 할 때 | 구현마다 `as_any` 우회 메서드 | `Any` 슈퍼트레이트와 업캐스트 | 2.5 |
| 모든 필드 훑기, 이름으로 필드 읽기 | 이름 문자열 → 값 함수 | derive, 접근 함수 표 | 2.6 |

`std::any`의 세 도구가 답할 수 있는 질문은 좁습니다.

- `type_name::<T>()`, `type_name_of_val(&v)`: 사람이 읽을 타입 이름. 컴파일할 때 정해지는 "최선을 다한" 문자열입니다.
- `TypeId`: 두 타입이 같은지 비교할 수 있는 식별자. `'static` 타입에만 있습니다.
- `dyn Any`: 값과 함께 `TypeId`를 들고 다녀서, 실행 중에 "정확히 이 구체 타입인가"를 확인하고 그 타입으로 꺼냅니다.

셋 다 "어떤 필드가 있나", "어떤 트레이트를 구현했나"에는 답하지 못합니다.

### 2.2 지침 1: `type_name`은 진단에만 쓴다

**문제 상황.** 학습 플랫폼의 활동 기록(이벤트)을 종류에 따라 다른 곳으로 보냅니다. 수강 신청(`Enrolled`)은 환영 메일로, 퀴즈 제출(`QuizSubmitted`)은 성적 처리로 갑니다. 예전 시스템에서 넘어온 수강 신청(`legacy::Enrolled`)은 옛 형식 변환으로 보내야 하고, 같은 이벤트를 묶은 `Batch<E>`도 들어옵니다. 다른 언어에서 하던 대로 타입 이름을 보고 경로를 고르고 싶어집니다.

**비권장 코드.** `type_name`의 출력을 한 번 찍어 보고, 그 문자열과 비교합니다.

```rust
fn route_by_exact_name<E>(_event: &E) -> &'static str {
    match type_name::<E>() {
        "01_type_name_limits::events::Enrolled" => "환영 메일",
        "01_type_name_limits::events::QuizSubmitted" => "성적 처리",
        _ => "버림",
    }
}
```

문자열에 크레이트 이름 `01_type_name_limits`(예제 파일 이름)와 모듈 경로가 들어 있다는 것부터 문제입니다. 타입을 다른 모듈로 옮기거나 코드를 라이브러리 크레이트로 떼어 내면 모든 갈래가 빗나갑니다. 그래서 끝부분만 비교하는 판(`name.ends_with("Enrolled")`)도 만들었습니다. [`01_type_name_limits.rs`](examples/01_type_name_limits.rs)의 2번 구역에서 두 판과 권장 코드를 같은 이벤트에 적용한 결과입니다.

```text
2. 처리 경로 고르기: 기대 | 전체 이름 비교 | 끝부분 비교 | 트레이트
    Enrolled: 기대 환영 메일 | 환영 메일 | 환영 메일 | 환영 메일
    QuizSubmitted: 기대 성적 처리 | 성적 처리 | 성적 처리 | 성적 처리
    &Enrolled: 기대 환영 메일 | 버림(틀림) | 환영 메일 | 환영 메일
    Batch<Enrolled>: 기대 환영 메일 | 버림(틀림) | 버림(틀림) | 환영 메일
    legacy::Enrolled: 기대 옛 형식 변환 | 버림(틀림) | 환영 메일(틀림) | 옛 형식 변환
```

- `&Enrolled`: 반복자에서 꺼낸 원소처럼 참조를 넘기면 `E`가 `&Enrolled`가 되어 이름 앞에 `&`가 붙습니다.
- `Batch<Enrolled>`: 제네릭 인자가 이름 안에 들어가 끝부분도 `Enrolled>`가 됩니다.
- `legacy::Enrolled`: 이름이 같은 다른 타입입니다. 끝부분 비교는 둘을 구별하지 못해 엉뚱한 곳으로 보냅니다.

모두 컴파일은 되고 실행 중에 조용히 틀립니다. 3번 구역은 이름에 아예 남지 않는 것도 보여 줍니다. 별칭 `type Minutes = u32`와 `type Score = u32`는 둘 다 `u32`로 나오고, `&'static str`과 지역 `String`을 빌린 `&str`은 다른 타입인데도 둘 다 `&str`로 나옵니다. 1.100.0의 `type_name` 문서는 다음을 밝힙니다.

- 진단용이며 내용과 형식은 정해져 있지 않다. 예를 들어 `Option<String>`의 이름으로 짧은 꼴과 전체 경로 꼴이 모두 가능하다.
- 서로 다른 타입이 같은 이름을 가질 수 있으므로 고유 식별자로 보면 안 된다. 타입의 모든 부분이 이름에 나온다는 보장도 없다.
- 컴파일러 버전에 따라 바뀔 수 있다(예전 버전에서는 라이프타임 표기가 빠졌다).

`type_name_of_val`(1.76.0)의 문서는 한 가지를 더 적습니다. 트레이트 객체를 넘기면 안에 든 구체 타입이 아니라 `dyn Trait`이라고 답합니다. ch12 예제 01의 `dyn Loanable`이 그 예입니다.

**권장 코드.** 처리 경로는 이벤트 타입이 답하게 합니다.

```rust
trait Event {
    fn route(&self) -> &'static str;
}

impl Event for Enrolled { fn route(&self) -> &'static str { "환영 메일" } }
impl Event for legacy::Enrolled { fn route(&self) -> &'static str { "옛 형식 변환" } }
// QuizSubmitted도 같은 모양

// 묶음은 안에 든 이벤트를 따르고, 참조는 가리키는 값을 따릅니다.
impl<E: Event> Event for Batch<E> { /* self.0.first().map_or("버림", Event::route) */ }
impl<E: Event + ?Sized> Event for &E { /* (**self).route() */ }
```

다섯 줄이 모두 맞습니다. 이름이 같은 두 타입은 `impl`이 따로 있으므로 섞일 수 없고, `Batch<E>`와 `&E`는 포괄 구현 하나로 모든 이벤트에 적용됩니다. `&E`에 대한 포괄 구현은 표준 라이브러리가 `Display`와 `Debug`에 해 둔 것과 같은 모양입니다. 무엇보다 `Event`를 구현하지 않은 타입은 `route_by_trait`에 넘길 수 없으므로 "모르는 타입이라 버림" 갈래가 생기지 않습니다. 실수는 실행 중이 아니라 컴파일할 때 드러납니다.

**같은 타입인지만 알면 될 때는 `TypeId`.** 이름 대신 `TypeId::of::<E>() == TypeId::of::<Enrolled>()`로 비교하면 모듈 경로 문제와 이름이 같은 다른 타입 문제는 사라집니다. 하지만 `Batch<Enrolled>`는 여전히 `Enrolled`와 다른 `TypeId`이고, `TypeId::of`는 `'static` 타입만 받으므로 지역 값을 빌린 `&Enrolled`는 비교할 수조차 없습니다(4.3절). 구체 타입 목록을 손으로 관리하는 문제가 남으므로 처리를 고르는 데는 트레이트가 낫습니다.

**지침을 따르지 않아도 되는 경우.** 로그, 에러 메시지, 테스트 실패 메시지, 디버깅 출력은 `type_name`의 원래 용도입니다. ch12가 단형성화를 관찰하는 데 쓴 것처럼 사람이 읽고 끝나는 곳이라면 마음껏 씁니다. 결과로 분기하거나, 파싱하거나, 저장해 두었다가 다음 실행에서 비교하지만 않으면 됩니다.

### 2.3 지침 2: 타입별 처리는 다운캐스트 사슬 대신 트레이트 메서드로

**문제 상황.** 강의 하나를 이루는 학습 자료를 `Vec<Box<dyn Lesson>>`에 담았습니다. 강의 전체의 예상 학습 시간과 오프라인으로 내려받을 크기를 구해야 합니다. 영상은 재생 시간, 퀴즈는 문항당 2분, 읽기 자료는 쪽당 3분이고, 내려받을 수 있는 것은 영상과 읽기 자료뿐입니다. 공통 트레이트에는 제목만 있고, 계산에 필요한 값은 타입마다 다른 필드에 있습니다.

**비권장 코드.** ch12 2.5절이 든 두 방법 중 다운캐스트를 고릅니다. 트레이트에 `Any`를 슈퍼트레이트로 두고(2.5절), `&dyn Any`로 올린 뒤 구체 타입을 하나씩 시도합니다.

```rust
pub fn study_minutes(item: &dyn Lesson) -> u32 {
    let any: &dyn Any = item;
    if let Some(v) = any.downcast_ref::<Video>() {
        v.minutes
    } else if let Some(q) = any.downcast_ref::<Quiz>() {
        q.questions * 2
    } else if let Some(r) = any.downcast_ref::<Reading>() {
        r.pages * 3
    } else {
        0 // 모르는 타입
    }
}
```

`download_mb`도 같은 모양의 사슬입니다. 처음 세 타입만 있을 때는 잘 동작합니다. 문제는 다른 팀이 실시간 특강 `LiveSession`을 더할 때 생깁니다. `impl Lesson for LiveSession`에 제목만 구현하면 컴파일되고, 다른 곳에 있는 두 함수를 고쳐야 한다는 사실은 아무도 알려 주지 않습니다. [`02_downcast_chain_vs_trait.rs`](examples/02_downcast_chain_vs_trait.rs)의 1번 구역에서 60분짜리 특강은 0분, 850MB 녹화본은 "다운로드 없음"이 되어 합계가 81분으로 나옵니다(기대 141분).

**권장 코드.** 필요한 동작을 트레이트 메서드로 둡니다. 모든 자료가 답해야 하는 질문은 기본 구현이 없는 필수 메서드로, 일부만 지원하는 기능은 기본 구현이 "지원하지 않음"을 돌려주는 선택적 메서드로 둡니다.

```rust
pub trait Lesson {
    fn title(&self) -> &str;
    fn study_minutes(&self) -> u32;          // 필수: 기본 구현 없음
    fn download_mb(&self) -> Option<u32> {   // 선택: 기본은 None
        None
    }
}

impl Lesson for LiveSession {
    fn title(&self) -> &str { &self.title }
    fn study_minutes(&self) -> u32 { self.minutes }
    fn download_mb(&self) -> Option<u32> { Some(self.recording_mb) }
}
```

2번 구역은 60분과 850MB를 바르게 반영해 141분을 냅니다. 두 판의 차이는 **새 타입을 더할 때 누가 알려 주는가**입니다.

| | `Any`판 | 트레이트판 |
| --- | --- | --- |
| 타입별 처리가 있는 곳 | 타입과 떨어진 함수의 `if let` 사슬 | 그 타입의 `impl` 블록 |
| 새 타입이 필수 동작을 빠뜨리면 | 경고 없이 `else` 갈래(0분) | E0046 ``not all trait items implemented, missing: `study_minutes` `` |
| 새 타입이 선택 기능을 빠뜨리면 | 경고 없이 `None` | 경고 없이 기본 구현(`None`) |
| 다른 크레이트가 타입을 더하면 | 사슬이 있는 크레이트를 고쳐야 함 | 타입을 만든 쪽이 구현 |
| 호출 비용 | 갈래마다 `TypeId` 비교 | vtable을 거친 호출 한 번 |

셋째 줄이 보여 주듯 기본 구현이 있는 메서드는 빠뜨려도 컴파일러가 알려 주지 않습니다. 그래서 기본 구현은 "빠뜨렸을 때 기본값이 맞는 답인가"로 정합니다. 대부분의 자료가 내려받기를 지원하지 않는다면 `None`이 맞는 기본값이고, 학습 시간처럼 모든 자료에 답이 있다면 기본 구현을 두지 않아 컴파일러가 묻게 합니다. 기본 구현을 언제 넉넉하게 둘지는 [ch13](../ch13-default-implementations/README.md)에서 다뤘습니다.

**닫힌 집합이라면 열거형.** 자료의 종류를 이 크레이트가 모두 정하고 다른 크레이트가 더할 일이 없다면 ch12 2.5절의 열거형도 선택지입니다. 3번 구역의 `Material`은 `Live(LiveSession)` 배리언트를 더하는 순간 `match`마다 E0004(``non-exhaustive patterns: `&Material::Live(_)` not covered``)로 고칠 곳을 알려 줍니다. 종류가 열려 있으면 트레이트, 닫혀 있으면 열거형이라는 기준은 ch12와 [ch1](../ch1-express-data-structures/README.md)에서 다뤘습니다.

**지침을 따르지 않아도 되는 경우.** 책은 `std::any`를 다른 방법이 없을 때만 쓰라고 합니다. 이 저장소에서 본 "다른 방법이 없는" 경우는 담는 쪽이 들어올 타입을 미리 알 수 없을 때입니다.

- 패닉 페이로드(`Box<dyn Any + Send>`)에서 메시지 꺼내기: [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md), Part 1 ch14 2.7절
- 저장소를 만드는 쪽이 타입을 모르는 타입 맵(`HashMap<TypeId, Box<dyn Any>>`): Part 1 ch14 2.8절
- `Box<dyn Error>`에서 특정 에러 골라내기: Part 1 ch8 2.5절, [ch4](../ch4-idiomatic-errors/README.md) 2.4절. ch4가 보였듯 라이브러리가 에러를 한 겹 감싸면 다운캐스트가 빗나가므로, 고를 일이 잦다면 에러 열거형을 받습니다.

이때도 다운캐스트는 "정확히 이 타입인가"만 묻고, 실패를 `None`이나 `Err`로 돌려받아 처리합니다.

### 2.4 지침 3: "이 기능도 있나?"는 `Any`가 아니라 트레이트로 묻는다

**문제 상황.** 채점할 수 있는 자료(`Gradable`, 만점을 돌려주는 `max_points`)의 만점을 더해 강의의 총점을 정합니다. 퀴즈가 `Gradable`을 구현하고, 나중에 코딩 과제 `CodingTask`도 구현합니다. 다른 언어라면 목록을 돌며 "이 객체가 `Gradable`도 구현했나?"를 실행 중에 물었을 것입니다.

**비권장 코드.** `&dyn Lesson`을 `&dyn Gradable`로 바꾸려 하면 E0308입니다(4.2절). 두 트레이트 사이에 슈퍼트레이트 관계가 없으면 컴파일러는 한쪽 vtable에서 다른 쪽 vtable을 찾을 방법이 없습니다. `Any`로도 이 질문은 할 수 없습니다. `std::any` 모듈 문서가 밝히듯 `&dyn Any`는 정해진 구체 타입인지만 확인할 수 있고, 어떤 트레이트를 구현했는지는 확인할 수 없습니다. 그래서 결국 아는 구체 타입을 하나씩 묻게 됩니다.

```rust
let any: &dyn Any = &**item; // &Box<dyn Lesson> → &dyn Lesson → &dyn Any
any.downcast_ref::<Quiz>().map_or(0, Gradable::max_points)
```

이 코드를 쓸 때 채점할 수 있는 타입은 `Quiz`뿐이었습니다. [`03_capability_query.rs`](examples/03_capability_query.rs)의 1번 구역에서 코딩 과제(60점)가 빠져 합계가 100으로 나옵니다(기대 160). "`Gradable`을 구현한 타입인가"라는 질문을 "이 목록에 있는 타입인가"로 바꿔 물었기 때문에, 목록 밖의 타입은 조용히 빠집니다.

**권장 코드 1: 기능 조회 메서드.** 트레이트에 "그 기능을 빌려 달라"는 메서드를 두고, 기능이 있는 타입만 자기 자신을 돌려줍니다.

```rust
trait Lesson: Any {   // `: Any`는 1번 구역의 비권장 코드용. 기능 조회 메서드에는 필요 없음
    fn title(&self) -> &str;
    fn as_gradable(&self) -> Option<&dyn Gradable> {
        None
    }
}

impl Lesson for CodingTask {
    fn title(&self) -> &str { &self.title }
    fn as_gradable(&self) -> Option<&dyn Gradable> {
        Some(self) // &CodingTask → &dyn Gradable 강제 변환
    }
}
```

2번 구역의 `items.iter().filter_map(|item| item.as_gradable())`에는 구체 타입이 하나도 없습니다. `Gradable`을 구현하는 새 타입은 자기 `impl Lesson`에 `Some(self)` 한 줄을 더하면 됩니다. 2.3절의 선택적 메서드와 같은 모양이라 같은 약점도 있어서, 그 한 줄을 빠뜨리면 기본 구현이 `None`을 돌려줍니다. 그래도 빠뜨린 곳이 그 타입의 `impl` 블록 안이므로 사슬처럼 다른 파일의 함수를 찾아다닐 필요는 없습니다.

**권장 코드 2: 처음부터 바운드로 요구.** 채점할 자료를 처음부터 따로 모을 수 있다면 실행 중에 물을 필요가 없습니다. ch12 2.4절의 조합 트레이트로 두 트레이트를 함께 요구합니다.

```rust
trait GradedLesson: Lesson + Gradable {}
impl<T: Lesson + Gradable> GradedLesson for T {}

let graded: Vec<Box<dyn GradedLesson>> = vec![Box::new(quiz()), Box::new(coding_task())];
```

이 목록에 영상을 넣으면 컴파일되지 않습니다. 제목만 필요한 함수 `title_of(item: &dyn Lesson)`에는 `&dyn GradedLesson`을 그대로 넘깁니다. 슈퍼트레이트 쪽으로 바꾸는 업캐스트가 일어나기 때문입니다(3번 구역).

**권장 코드 3: 메서드가 없는 성질은 마커 트레이트.** "수료 요건에 들어가는 자료"처럼 메서드로 표현할 것이 없는 성질은, 다른 언어라면 클래스 이름 규칙이나 실행 중에 읽는 표시(애너테이션)로 했을 것입니다. 책은 이런 성질을 [ch2](../ch2-express-common-behavior/README.md) 2.5절의 마커 트레이트로 표현하라고 권합니다.

```rust
trait Certified: Lesson {}
impl Certified for Quiz {}
impl Certified for CodingTask {}

let requirements: [&dyn Certified; 2] = [&q, &c];
```

4번 구역의 배열에 영상을 더하면 ``the trait bound `Video: Certified` is not satisfied``(E0277)입니다. 슈퍼트레이트 `Lesson`의 `title()`은 `&dyn Certified`에서 바로 부를 수 있습니다.

**셋 중 무엇을 고를까.** 한 목록 안에서 자료마다 답이 다르면 기능 조회 메서드, 기능이 있는 자료만 모아 다루는 코드라면 바운드, 메서드 없는 약속이면 마커 트레이트입니다. 바운드와 마커 트레이트는 컴파일할 때 확인되므로 가능하면 이쪽을 먼저 봅니다.

**지침을 따르지 않아도 되는 경우.** 트레이트가 다른 크레이트의 것이라 기능 조회 메서드를 더할 수 없다면, 내 쪽에 그 트레이트를 감싸는 트레이트나 열거형을 만들거나 기능이 있는 값을 처음부터 따로 담습니다. 구체 타입이 몇 개로 정해져 있고 바뀔 일이 없다면 다운캐스트로도 결과는 맞지만, 그런 닫힌 집합에는 열거형이 더 낫습니다(2.3절).

### 2.5 지침 4: 다운캐스트가 꼭 필요하면 `Any`를 슈퍼트레이트로 두고 업캐스트한다

**문제 상황.** 2.3절의 예외에 해당해서 내 트레이트 객체를 정말 구체 타입으로 꺼내야 한다고 합시다. `dyn Lesson`에서 `Quiz`를 꺼내려면 `&dyn Any`가 필요합니다.

**비권장 코드(책을 쓸 때의 모양).** 책을 쓸 때(rustc 1.70)는 트레이트 객체를 다른 트레이트 객체로 바꿀 수 없어서, 트레이트에 `&dyn Any`를 돌려주는 메서드를 두고 구현마다 같은 한 줄을 적었습니다. 지금도 오래된 코드에서 자주 봅니다.

```rust
pub trait Lesson {
    fn title(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

impl Lesson for Video {
    fn title(&self) -> &str { &self.title }
    fn as_any(&self) -> &dyn Any { self } // 모든 구현에 똑같이
}
```

**권장 코드.** 1.86.0부터는 트레이트 객체를 슈퍼트레이트의 트레이트 객체로 바꾸는 업캐스트가 강제 변환으로 일어납니다([ch5](../ch5-type-conversions/README.md) 2.6절, ch12 2.4절). `Any`를 슈퍼트레이트로 두면 `&dyn Lesson`을 `&dyn Any`로 바로 바꿀 수 있습니다.

```rust
trait Lesson: Any {
    fn title(&self) -> &str;
}

let lesson: &dyn Lesson = item.as_ref();
let any: &dyn Any = lesson; // 업캐스트. 우회 메서드가 필요 없음
let questions = any.downcast_ref::<Quiz>().map(|q| q.questions);
```

[`04_upcast_to_any.rs`](examples/04_upcast_to_any.rs)의 1번 구역(`as_any`)과 2번 구역(업캐스트)은 같은 결과를 냅니다. 현재 업캐스트의 동작을 정리하면 다음과 같습니다. 모두 이 툴체인에서 확인했습니다.

- **슈퍼트레이트 쪽으로만 됩니다.** `trait GradedLesson: Lesson + Gradable`이고 `trait Lesson: Any`이면 `&dyn GradedLesson` → `&dyn Any`처럼 여러 단계도 한 번에 됩니다(3번 구역). 슈퍼트레이트 관계가 없는 `&dyn Lesson` → `&dyn Gradable`은 E0308이고(4.2절), `as`로 적어도 E0605(``non-primitive cast``)입니다.
- **한 방향으로만 됩니다.** `Box<dyn Lesson>`을 `Box<dyn Any>`로 올리면 `downcast`로 구체 타입을 소유권째 꺼낼 수 있습니다. 하지만 실패해서 돌려받은 상자는 `Box<dyn Any>`이고, 이것을 `Box<dyn Lesson>`으로 되돌리면 E0308입니다(4번 구역). 실패한 뒤에도 원래 트레이트가 필요하다면, 먼저 `&dyn Any`로 빌려 `is::<T>()`로 확인한 다음에 상자를 올립니다.
- **올려야 다운캐스트 메서드가 생깁니다.** `downcast_ref`와 `is`는 `dyn Any`에만 있는 고유 메서드라서, `Any`를 슈퍼트레이트로 두어도 `&dyn Lesson`에서 바로 부르면 E0599입니다(4.1절).
- **`type_id()`는 부르는 대상에 주의합니다.** `&dyn Lesson`에서 부르면 안에 든 구체 타입의 `TypeId`가 나옵니다. `Any::type_id` 문서도 `dyn Any`와 그 하위 트레이트의 트레이트 객체에서는 그렇다고 적습니다. 그러나 `Box<dyn Lesson>`에서 바로 부르면 상자 자체의 `TypeId`가 나옵니다(5번 구역). Part 1 ch14 2.7절의 "상자 자체도 `Any`" 함정과 같고, clippy의 `type_id_on_box`가 `Box<dyn Lesson>`에서도 경고합니다. 5번 구역은 일부러 보여 주려고 `#[expect]`로 막았습니다.

업캐스트가 들어왔다고 해서 책의 결론이 바뀌지는 않습니다. `dyn Any`에서는 여전히 정확한 구체 타입만 물을 수 있고 다른 트레이트로 갈 길은 없습니다. 바뀐 것은 다운캐스트가 꼭 필요할 때의 준비 코드뿐입니다.

**지침을 따르지 않아도 되는 경우.** 1.86.0보다 오래된 툴체인도 지원해야 하는 라이브러리라면 `as_any` 모양을 유지합니다(지원 버전을 정하는 일은 [ch21](../ch21-semver/README.md)에서 다룹니다). 또 `Any`를 슈퍼트레이트로 두면 그 트레이트의 모든 구현이 `'static`이어야 합니다. 빌린 데이터를 품은 `Preview<'a>`에 `impl<'a> Lesson for Preview<'a>`를 쓰면 E0478(``lifetime bound not satisfied``)입니다. 그런 타입도 구현해야 하는 트레이트라면 다운캐스트를 포기하고 2.3·2.4절의 방법을 씁니다.

### 2.6 지침 5: 필드를 훑는 코드는 derive에 맡긴다

**문제 상황.** 수강 기록 `Enrollment`(수강생, 강의, 진도율)를 쉼표로 구분한 줄로 내보내고, 디버깅할 때는 모든 필드를 찍어 봅니다. 나중에 수료증 발급 여부 `certificate` 필드를 더합니다. 다른 언어라면 리플렉션으로 필드 목록을 훑거나 이름으로 필드를 찾았을 일입니다.

**비권장 코드.** 필드 이름 문자열을 받아 값을 돌려주는 함수를 손으로 만듭니다.

```rust
fn field(&self, name: &str) -> Option<String> {
    match name {
        "learner" => Some(self.learner.clone()),
        "course" => Some(self.course.clone()),
        "progress" => Some(self.progress.to_string()),
        _ => None, // certificate를 더할 때 이 함수는 고치지 않음
    }
}
```

[`05_derive_over_field_lookup.rs`](examples/05_derive_over_field_lookup.rs)의 1번 구역은 `["learner", "progres", "certificate"]` 열을 요청합니다. 오타 `"progres"`와 이 함수가 모르는 새 필드 `"certificate"`가 모두 `None`이 되어 `?`로 나옵니다. 필드 이름이 컴파일러가 모르는 문자열이 된 순간, 오타와 빠뜨림은 실행 결과로만 드러납니다.

**권장 코드 1: 모든 필드를 보여 주는 일은 derive에.** `#[derive(Debug)]`는 컴파일할 때 구조체 정의를 읽어 `Debug` 구현을 만듭니다. `Debug` 문서는 derive한 구현이 구조체 이름과 필드마다 이름과 디버그 형식의 값을 출력한다고 설명하고, 같은 출력을 내는 손으로 쓴 구현(`f.debug_struct("..").field("x", &self.x)...`)을 함께 보여 줍니다. 필드를 더하면 다음 컴파일에서 구현도 바뀌므로 2번 구역의 출력에는 `certificate`가 저절로 들어 있습니다. 실행 중에 필드를 찾는 일은 없습니다.

책이 리플렉션의 대안으로 드는 것이 이것입니다. 매크로, 특히 derive 매크로는 타입을 컴파일할 때 이해하는 코드를 만들어 주므로, 실행 중에 내용을 해석하는 것보다 효율적이고 타입 검사도 받습니다. `Clone`, `PartialEq` 같은 표준 derive도 필드마다 일하는 코드를 같은 방식으로 만들고, 직렬화처럼 필드를 훑어야 하는 다른 일도 생태계에서는 derive로 풉니다(대표적인 크레이트가 `serde`입니다). [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.20절의 `hello_macro`가 derive를 붙인 타입의 이름을 컴파일할 때 `stringify!`로 문자열로 넣은 것도 같은 원리입니다. derive 매크로를 직접 만드는 일은 [`ch28-macros-judiciously`](../ch28-macros-judiciously/README.md)와 Part 9 `ch2-macros`에서 다룹니다.

다만 `Debug` 출력도 사람이 읽는 용도입니다. 문서는 derive한 `Debug`의 형식이 안정적이지 않아 Rust 버전에 따라 바뀔 수 있다고 적으므로, `type_name`처럼 파싱하지 않습니다.

**권장 코드 2: 고른 열만 내보낼 때는 접근 함수 표.** 열 이름과 값을 꺼내는 함수를 짝지어 둡니다.

```rust
type Column = (&'static str, fn(&Enrollment) -> String);

const COLUMNS: [Column; 3] = [
    ("learner", |e| e.learner.clone()),
    ("progress", |e| e.progress.to_string()),
    ("certificate", |e| e.certificate.to_string()),
];
```

머리글은 그냥 문자열이지만 값을 꺼내는 쪽은 보통의 필드 접근입니다. `e.progres`라고 쓰면 E0609(``no field `progres` on type `&Enrollment` ``)로 컴파일되지 않고, 도움말이 `progress`를 제안합니다. 캡처하지 않는 클로저가 함수 포인터로 바뀌는 강제 변환(ch2 2.1절, ch5 2.6절)을 쓴 모양입니다. 3번 구역이 결과입니다.

**지침을 따르지 않아도 되는 경우.** 열 이름이 명령줄 인자나 설정 파일 같은 사용자 입력으로 들어온다면 문자열을 어딘가에서 필드로 바꿔야 합니다. 그 변환은 입력을 받는 경계 한 곳에서만 하고([ch5](../ch5-type-conversions/README.md)의 "경계"), 모르는 이름은 `None`으로 삼키지 말고 에러로 알립니다. 위의 `COLUMNS`에서 머리글이 같은 열을 찾고, 없으면 `Err`를 돌려주는 식입니다(연습 3).

### 2.7 책과 달라진 점

책은 2018 에디션, rustc 1.70을 기준으로 합니다. 이 Item의 본문은 2021년에 쓰였고, 출간 전에 업캐스트 소식을 덧붙였습니다. rustc 1.100.0과 로컬 문서에서 확인한 차이는 다음과 같습니다.

- **업캐스트는 1.86.0에서 안정화되었습니다.** 책은 업캐스트가 기능 게이트 뒤에 있고 1.76에 들어올 예정이라고 적었습니다. 실제 안정화는 1.86.0이었고, 1.100.0에서는 `&dyn Lesson` → `&dyn Any`, `Box<dyn Lesson>` → `Box<dyn Any>`, 여러 단계 업캐스트가 모두 강제 변환으로 됩니다(2.5절).
- **`Any`의 하위 트레이트 객체에서 다운캐스트할 수 있습니다.** 책은 `Any`에 슈퍼트레이트가 없으므로 업캐스트가 `Any`에 관한 조언을 바꾸지 않는다고 했습니다. `dyn Any`에서 다른 트레이트로 갈 수 없다는 점은 그대로입니다. 그러나 반대 방향인 `trait Lesson: Any`의 `&dyn Lesson` → `&dyn Any`는 되므로, 다운캐스트용 `as_any` 우회 메서드는 필요 없어졌습니다. 1.86.0 발표 글도 이 쓰임을 소개합니다.
- **`type_name_of_val`이 1.76.0에서 안정화되었습니다.** 책은 `type_name`을 감싼 도우미 함수를 직접 만들어 값의 타입 이름을 얻었습니다. 지금은 같은 일을 하는 표준 함수가 있고, 문서는 트레이트 객체를 넘기면 `dyn Trait`이라고만 답한다고 밝힙니다.
- **문서의 주의 문구.** `type_name` 문서는 고유 식별자가 아니고, 타입의 모든 부분이 이름에 나온다는 보장이 없으며, 컴파일러 버전마다 바뀔 수 있다고 적습니다. `TypeId` 문서는 지금은 `'static` 타입에만 쓸 수 있지만 이 제한은 나중에 없어질 수 있고, 해시와 순서가 Rust 릴리스마다 달라질 수 있다고 적습니다. `std::any` 모듈 문서는 `&dyn Any`로는 트레이트를 구현했는지 확인할 수 없다고 명시합니다.
- **`'static` 조건을 어긴 제네릭 코드는 E0310입니다.** 제네릭 함수에서 `TypeId::of::<T>()`를 부르거나 `&T`를 `&dyn Any`로 바꾸면 ``the parameter type `T` may not live long enough``와 함께 `T: 'static`을 더하라는 도움말이 나옵니다(4.3절). 책이 가리키는 이슈(rust-lang/rust#41875)는 이 제한의 이유를 다룹니다.
- **`TypeId::of`를 `const`에서 쓸 수 있습니다.** 로컬 문서에 따르면 1.91.0부터 `const fn`이고, `const ID: TypeId = TypeId::of::<u32>();`가 컴파일됩니다.
- **nightly 전용 실험이 문서에 보입니다.** 1.100.0 문서에는 값이 어떤 트레이트 객체로 바뀔 수 있으면 `Some`을 돌려주는 `std::any::try_as_dyn`과, 컴파일할 때 필드 수나 배리언트 이름 같은 타입 정보를 꺼내는 `TypeId`의 메서드들(`type_info` 기능)이 nightly 전용 실험 API로 있습니다. 안정 툴체인에서 쓰면 E0658입니다. `try_as_dyn` 문서는 구현했는데도 `None`을 돌려줄 수 있으니 결과의 정확성에 기대지 말라고 적습니다. 안정 Rust에는 여전히 리플렉션이 없고, 이 챕터의 지침도 그대로입니다.
- **책에 없는 지침.** 다음은 책의 Item 19가 명시적으로 권하지 않는 내용입니다(2026-11-19에 온라인판에서 확인).
  - 2.3절의 열거형판: 책은 대안으로 트레이트, 마커 트레이트, 매크로만 듭니다. ch12 2.5절, ch1, Part 0 ch17 2.10절이 근거입니다.
  - 2.3·2.4절의 선택적 메서드(기본 구현으로 `None`)와 기능 조회 메서드 `as_gradable`, 기본 구현을 둘지 정하는 기준: 책은 "필요한 동작을 트레이트로 표현하라"고만 합니다. Part 0 ch17 2.10절의 `fn number(&self) -> Option<f64> { None }`과 ch13이 근거입니다.
  - 2.3절의 "`Any`가 정당한 경우" 목록: 책은 "다른 방법이 없을 때만"이라고만 합니다. Part 1 ch8·ch14와 ch4가 다룬 쓰임을 모았습니다.
  - 2.5절의 `as_any` 대신 업캐스트하기와 `Any` 슈퍼트레이트의 `'static` 제약: 1.86.0 발표 글과 이 툴체인에서 컴파일한 결과(E0478 포함)가 근거입니다.
  - 2.6절의 접근 함수 표와 "사용자 입력은 경계에서 에러로": 책은 derive 매크로까지만 말합니다. 필드 이름을 컴파일러가 확인한다는 점(E0609)과 ch5의 경계 개념이 근거입니다.

### 2.8 기억할 것

- Rust에는 리플렉션이 없습니다. 실행 중에 타입을 묻는 대신 컴파일할 때 답이 정해지는 트레이트, 열거형, derive로 풉니다.
- `type_name`은 사람이 읽는 진단용입니다. 결과로 분기하거나 파싱하지 않습니다.
- 타입별 처리는 그 타입의 `impl`에 둡니다. 모든 타입이 답해야 하면 필수 메서드, 일부만 지원하면 기본 구현이 있는 선택적 메서드, 닫힌 집합이면 열거형입니다.
- `Any`는 "정확히 이 구체 타입인가"만 답합니다. "이 기능도 있나"는 기능 조회 메서드, 바운드, 마커 트레이트로 묻습니다.
- 다운캐스트가 꼭 필요하면 `Any`를 슈퍼트레이트로 두고 `&dyn Any`로 업캐스트합니다. 업캐스트는 슈퍼트레이트 쪽으로, 한 방향으로만 됩니다.

## 3. 예제 안내

모든 예제는 같은 파일 안에서 비권장 형태와 권장 형태를 차례로 실행해 결과를 비교합니다. 저장소 루트에서 실행하며, 모든 예제는 종료 코드 0으로 끝납니다.

| 파일 | 지침 | 비교하는 것 | 관련 절 |
| --- | --- | --- | --- |
| [`01_type_name_limits.rs`](examples/01_type_name_limits.rs) | 1 | `type_name` 문자열 비교 두 가지와 트레이트 `Event` | 2.2 |
| [`02_downcast_chain_vs_trait.rs`](examples/02_downcast_chain_vs_trait.rs) | 2 | `Any`판, 트레이트판, 열거형판 | 2.3 |
| [`03_capability_query.rs`](examples/03_capability_query.rs) | 3 | 아는 타입만 다운캐스트, 기능 조회 메서드, 바운드, 마커 트레이트 | 2.4 |
| [`04_upcast_to_any.rs`](examples/04_upcast_to_any.rs) | 4 | `as_any` 우회 메서드와 업캐스트, 업캐스트의 동작 | 2.5 |
| [`05_derive_over_field_lookup.rs`](examples/05_derive_over_field_lookup.rs) | 5 | 이름 문자열로 필드 읽기, derive(`Debug`), 접근 함수 표 | 2.6 |

### 01_type_name_limits — `type_name`으로 분기하면

1번 구역은 `type_name`이 돌려준 문자열을 그대로 보여 줍니다. 2번 구역은 전체 이름 비교, 끝부분 비교, 트레이트로 처리 경로를 골라 기대값과 비교하고, 틀린 결과에 `(틀림)`을 붙입니다. 3번 구역은 별칭과 라이프타임이 이름에 남지 않는다는 것을 보여 줍니다. 1번과 3번 구역의 문자열은 툴체인에 따라 달라질 수 있습니다.

```sh
cargo run -p er-ch19-avoid-reflection --example 01_type_name_limits
```

```text
1. type_name이 돌려준 문자열 (진단용, 툴체인에 따라 다를 수 있음)
    01_type_name_limits::events::Enrolled
    &01_type_name_limits::events::Enrolled
    01_type_name_limits::Batch<01_type_name_limits::events::Enrolled>
    01_type_name_limits::events::legacy::Enrolled
2. 처리 경로 고르기: 기대 | 전체 이름 비교 | 끝부분 비교 | 트레이트
    Enrolled: 기대 환영 메일 | 환영 메일 | 환영 메일 | 환영 메일
    QuizSubmitted: 기대 성적 처리 | 성적 처리 | 성적 처리 | 성적 처리
    &Enrolled: 기대 환영 메일 | 버림(틀림) | 환영 메일 | 환영 메일
    Batch<Enrolled>: 기대 환영 메일 | 버림(틀림) | 버림(틀림) | 환영 메일
    legacy::Enrolled: 기대 옛 형식 변환 | 버림(틀림) | 환영 메일(틀림) | 옛 형식 변환
3. 이름에 남지 않는 것
    type Minutes = u32 → u32, type Score = u32 → u32
    &'static str → &str, 지역 String을 빌린 &str → &str
```

### 02_downcast_chain_vs_trait — 다운캐스트 사슬과 트레이트 메서드

세 구역 모두 같은 네 자료(영상, 퀴즈, 읽기 자료, 나중에 더한 실시간 특강)를 다룹니다. 1번 구역의 `Any`판은 특강을 모르는 타입으로 처리해 0분, "다운로드 없음"으로 계산합니다. 2번 구역의 트레이트판과 3번 구역의 열거형판은 특강을 바르게 반영합니다. 학습 시간은 영상 25분, 퀴즈 10문항 × 2분, 읽기 자료 12쪽 × 3분, 특강 60분입니다.

```sh
cargo run -p er-ch19-avoid-reflection --example 02_downcast_chain_vs_trait
```

```text
1. Any판: 구체 타입을 하나씩 다운캐스트
    1주차 강의 영상: 25분, 다운로드 180MB
    1주차 퀴즈: 20분, 다운로드 없음
    읽기 자료: 36분, 다운로드 4MB
    실시간 특강: 0분, 다운로드 없음
    합계 81분
2. 트레이트판: 각 타입이 메서드로 답함
    1주차 강의 영상: 25분, 다운로드 180MB
    1주차 퀴즈: 20분, 다운로드 없음
    읽기 자료: 36분, 다운로드 4MB
    실시간 특강: 60분, 다운로드 850MB
    합계 141분
3. 열거형판: 닫힌 집합이라면 match로
    합계 141분, 내려받을 것: 1주차 강의 영상 180MB, 읽기 자료 4MB, 실시간 특강 850MB
```

### 03_capability_query — "이 기능도 있나?" 묻기

목록은 영상, 퀴즈(10문항 × 10점), 코딩 과제(테스트 3개 × 20점)입니다. 1번 구역은 `Quiz`만 아는 다운캐스트로 코딩 과제를 빠뜨리고, 2번 구역은 기능 조회 메서드 `as_gradable`로 바르게 셉니다. 3번 구역은 채점할 자료만 `Vec<Box<dyn GradedLesson>>`에 담고, 제목은 `&dyn Lesson`으로 업캐스트해 얻습니다. 4번 구역은 마커 트레이트 `Certified`를 구현한 자료만 담은 배열입니다.

```sh
cargo run -p er-ch19-avoid-reflection --example 03_capability_query
```

```text
1. Any판: 아는 구체 타입(Quiz)만 확인
    만점 합계 100 (기대 160)
2. 기능 조회 메서드: item.as_gradable()
    1주차 강의 영상: 채점 없음
    1주차 퀴즈: 채점 가능, 만점 100
    1주차 코딩 과제: 채점 가능, 만점 60
    만점 합계 160
3. 처음부터 바운드로: Vec<Box<dyn GradedLesson>>
    1주차 퀴즈, 1주차 코딩 과제 → 만점 합계 160
4. 마커 트레이트: 수료 요건
    1주차 퀴즈, 1주차 코딩 과제
```

### 04_upcast_to_any — 업캐스트의 현재 동작

1번 구역은 `as_any` 우회 메서드, 2번 구역은 `&dyn Lesson` → `&dyn Any` 업캐스트로 같은 결과를 냅니다. 3번 구역은 두 단계를 건너뛰는 업캐스트, 4번 구역은 `Box<dyn Any>`로 올려 소유권째 꺼내기와 실패했을 때 돌려받는 상자, 5번 구역은 `type_id()`를 참조와 상자에서 부른 차이입니다.

```sh
cargo run -p er-ch19-avoid-reflection --example 04_upcast_to_any
```

```text
1. 예전 모양: 구현마다 as_any()를 적음
    1주차 강의 영상 → Video
    1주차 퀴즈 → Quiz
2. 지금 모양: &dyn Lesson → &dyn Any 업캐스트
    1주차 강의 영상 → Video, 문항 수 None
    1주차 퀴즈 → Quiz, 문항 수 Some(10)
3. 여러 단계 업캐스트: &dyn GradedLesson → &dyn Any
    1주차 퀴즈 (만점 100) → Quiz
4. Box째 올리고 downcast로 소유권째 꺼내기
    Quiz가 아니라 Box<dyn Any>로 돌려받음
    Video로 다시 시도해 꺼냄: 1주차 강의 영상
5. type_id(): 무엇에 부르느냐에 따라 답이 다름
    &dyn Lesson에서: Quiz와 같은가 true
    Box<dyn Lesson>에서: Quiz와 같은가 false
```

### 05_derive_over_field_lookup — 필드 이름 대신 derive와 접근 함수

1번 구역은 오타와 새 필드가 `?`로 조용히 사라지는 이름 조회, 2번 구역은 새 필드까지 저절로 보여 주는 `{:?}`, 3번 구역은 접근 함수 표로 고른 열입니다. 마지막 줄은 비교를 위해 `field()`가 아는 이름을 넣으면 값이 나온다는 것을 보여 줍니다.

```sh
cargo run -p er-ch19-avoid-reflection --example 05_derive_over_field_lookup
```

```text
1. 비권장: 필드 이름 문자열로 꺼내기
    learner,progres,certificate
    김하나,?,?
    이두리,?,?
2. 권장: derive(Debug)가 만든 코드는 새 필드도 함께 보여 줌
    Enrollment { learner: "김하나", course: "러스트 입문", progress: 80, certificate: false }
    Enrollment { learner: "이두리", course: "러스트 입문", progress: 100, certificate: true }
3. 권장: (머리글, 접근 함수) 표로 고른 열만 내보내기
    learner,progress,certificate
    김하나,80,false
    이두리,100,true
    (참고) field("course") = Some("러스트 입문")
```

## 4. 자주 하는 실수

셋 모두 "실행 중에 타입을 물어보면 되겠지"라는 기대를 컴파일러가 막는 경우입니다. 직접 확인하려면 챕터 디렉터리에서 다음을 실행합니다(`--out-dir`은 고쳐서 컴파일에 성공했을 때 산출물을 챕터 밖에 두기 위한 것입니다).

```sh
cd appd-a/ch19-avoid-reflection
rustc --edition 2024 compile_fail/e0599_downcast_on_subtrait_object.rs --out-dir /tmp/er-ch19
rustc --edition 2024 compile_fail/e0308_unrelated_trait_object.rs --out-dir /tmp/er-ch19
rustc --edition 2024 compile_fail/e0310_type_id_in_generic.rs --out-dir /tmp/er-ch19
```

### 4.1 `Any`의 하위 트레이트 객체에서 바로 다운캐스트 — E0599

[`compile_fail/e0599_downcast_on_subtrait_object.rs`](compile_fail/e0599_downcast_on_subtrait_object.rs)

```text
error[E0599]: no method named `downcast_ref` found for reference `&(dyn Lesson + 'static)` in the current scope
  --> compile_fail/e0599_downcast_on_subtrait_object.rs:28:10
   |
28 |     item.downcast_ref::<Quiz>().map(|q| q.questions) // E0599
   |          ^^^^^^^^^^^^ method not found in `&(dyn Lesson + 'static)`
```

- **읽는 법**: `trait Lesson: Any`라고 썼으니 `dyn Lesson`도 `Any`의 메서드를 다 가졌으리라 기대했지만, 컴파일러는 `&dyn Lesson`에서 `downcast_ref`를 찾지 못했습니다. 도움말도 없습니다.
- **원인**: `downcast_ref`, `downcast_mut`, `is`는 `Any` 트레이트의 메서드가 아니라 `dyn Any` 타입(과 `dyn Any + Send` 등)에 붙은 고유 메서드입니다. `Any` 트레이트 자체에 있는 메서드는 `type_id` 하나뿐입니다. 슈퍼트레이트는 그 트레이트의 메서드를 쓸 수 있게 해 줄 뿐 `dyn Any`의 고유 메서드까지 옮겨 주지 않습니다. `Any`를 슈퍼트레이트로 두지 않았을 때도 같은 E0599가 나며(메시지의 타입이 `&dyn Lesson`), 그때는 아래 해결책도 쓸 수 없습니다.
- **해결**: 먼저 업캐스트합니다. `let any: &dyn Any = item;` 다음에 `any.downcast_ref::<Quiz>()`를 부르거나, 식 안에서 `(item as &dyn Any).downcast_ref::<Quiz>()`로 씁니다(2.5절). 다만 그 전에, 정말 구체 타입을 꺼내야 하는지 2.3절의 트레이트 메서드로 충분한지부터 봅니다.

### 4.2 관계없는 트레이트 객체로 바꾸려 함 — E0308

[`compile_fail/e0308_unrelated_trait_object.rs`](compile_fail/e0308_unrelated_trait_object.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_unrelated_trait_object.rs:36:35
   |
36 |     let gradable: &dyn Gradable = item; // E0308
   |                   -------------   ^^^^ expected trait `Gradable`, found trait `Lesson`
   |                   |
   |                   expected due to this
   |
   = note: expected reference `&dyn Gradable`
              found reference `&dyn Lesson`
```

- **읽는 법**: `&dyn Lesson`을 `&dyn Gradable` 자리에 넣을 수 없다는 뜻입니다. 안에 든 `Quiz`가 `Gradable`을 구현했다는 사실은 메시지에 나오지 않습니다. 컴파일러는 그 사실을 이 자리에서 알 수 없기 때문입니다.
- **원인**: `&dyn Lesson`이 된 순간 구체 타입 정보는 vtable 뒤로 숨고, 컴파일러가 아는 것은 "`Lesson`을 구현한 무언가"뿐입니다. 업캐스트는 슈퍼트레이트 쪽으로만 되는데 `Gradable`은 `Lesson`의 슈퍼트레이트가 아닙니다. 실행 중에 "`Gradable`도 구현했나"를 물을 수단도 없습니다(2.4절). `item as &dyn Gradable`로 적으면 E0605(``non-primitive cast: `&dyn Lesson` as `&dyn Gradable` ``)입니다.
- **해결**: 트레이트에 기능 조회 메서드 `fn as_gradable(&self) -> Option<&dyn Gradable>`을 두거나, 처음부터 `Lesson + Gradable`을 함께 요구하는 조합 트레이트(`dyn GradedLesson`)로 받습니다(예제 03의 2·3번 구역). 모든 `Lesson`이 채점할 수 있어야 한다면 `trait Lesson: Gradable`로 슈퍼트레이트를 두는 것도 방법입니다.

### 4.3 제네릭 함수 안에서 `TypeId::of::<T>()` — E0310

[`compile_fail/e0310_type_id_in_generic.rs`](compile_fail/e0310_type_id_in_generic.rs)

```text
error[E0310]: the parameter type `T` may not live long enough
  --> compile_fail/e0310_type_id_in_generic.rs:28:8
   |
28 |     if TypeId::of::<T>() == TypeId::of::<Quiz>() {
   |        ^^^^^^^^^^^^^^^
   |        |
   |        the parameter type `T` must be valid for the static lifetime...
   |        ...so that the type `T` will meet its required lifetime bounds
   |
help: consider adding an explicit lifetime bound
   |
27 | fn label<T: Lesson + 'static>(item: &T) -> String {
   |                    +++++++++
```

- **읽는 법**: `TypeId::of`는 `T: 'static`을 요구하는데, `label`의 `T`에는 그런 조건이 없다는 뜻입니다. ``may not live long enough``는 `T`가 짧게 빌린 참조를 품은 타입일 수도 있다는 말입니다.
- **원인**: `TypeId`에는 라이프타임이 들어가지 않습니다. `'static`이 아닌 타입까지 허락하면, 잠깐 빌린 값을 담은 타입과 `'static`인 같은 모양의 타입을 구별하지 못해 다운캐스트로 라이프타임을 속일 수 있게 됩니다(Part 1 ch14 2.7절). 그 E0597은 지역 값을 빌린 구체 타입에서 났고, 여기서는 제네릭 매개변수라서 E0310입니다. `&T`를 `&dyn Any`로 바꾸려 해도 같은 E0310입니다.
- **해결**: 도움말대로 `T: Lesson + 'static`을 붙이면 컴파일되고 `[퀴즈] 1주차 퀴즈`를 출력합니다. 하지만 그러면 `label`은 빌린 데이터를 품은 타입(`Preview<'a>` 같은)을 받지 못합니다. 지역 `String`을 빌린 `Preview`를 넘기면 ``argument requires that `text` is borrowed for `'static` ``(E0597)입니다. 더 근본적으로는 "제네릭 안에서 특정 타입만 특별 취급"이 리플렉션 습관입니다. 제목 앞에 붙일 꼬리표를 `fn badge(&self) -> Option<&'static str> { None }` 같은 선택적 메서드로 두면 `'static` 조건도 `TypeId`도 필요 없습니다(연습 2).

### 4.4 그 밖의 에러

| 상황 | 에러 | 해결 |
| --- | --- | --- |
| 관계없는 트레이트 객체로 `as` 캐스트 (`item as &dyn Gradable`) | E0605 (``non-primitive cast``) | 기능 조회 메서드나 조합 트레이트(2.4절) |
| 다운캐스트에 실패한 `Box<dyn Any>`를 `Box<dyn Lesson>`으로 되돌림 | E0308 (``expected trait `Lesson`, found trait `Any` ``) | 상자를 올리기 전에 `&dyn Any`로 빌려 `is::<T>()`로 확인(2.5절) |
| `trait Lesson: Any`를 빌린 데이터를 품은 타입에 구현 | E0478 (``lifetime bound not satisfied``) | 다운캐스트 대신 트레이트 메서드(2.5절) |
| 마커 트레이트를 구현하지 않은 값을 `&dyn Certified`로 | E0277 (``the trait bound `Video: Certified` is not satisfied``) | 정말 그 성질이 있는지 확인하고 `impl`을 더하거나, 넣지 않음(2.4절) |
| 새 타입이 필수 메서드를 빠뜨림 | E0046 (``missing: `study_minutes` ``) | 이 에러가 트레이트판의 장점입니다. 메서드를 구현합니다(2.3절). |
| 열거형에 배리언트를 더하고 `match`를 고치지 않음 | E0004 (``non-exhaustive patterns``) | 갈래를 더합니다(2.3절). |
| 접근 함수 안의 필드 이름 오타 | E0609 (``no field `progres` on type `&Enrollment` ``) | 도움말이 제안하는 이름으로 고칩니다(2.6절). |
| `try_as_dyn`이나 `TypeId::size()` 같은 실험 API | E0658 (``use of unstable library feature``) | 안정 툴체인에서는 쓸 수 없습니다(2.7절). |

## 5. 연습 문제

### 연습 1. 과제를 추가하면

예제 02에 새 자료 `Assignment`(예상 학습 90분, 내려받을 것 없음)를 세 판 모두에 더해 보세요. 판마다 고친 곳의 수와, 고치는 것을 잊었을 때 무슨 일이 생기는지 비교해 보세요.

<details>
<summary>힌트</summary>

`Any`판은 `impl Lesson for Assignment`에 제목만 쓰면 컴파일되고, `study_minutes`의 사슬에 갈래를 더하지 않으면 0분이 됩니다. 컴파일러는 아무 말도 하지 않습니다. 트레이트판은 `impl Lesson for Assignment`에 `study_minutes`를 쓰지 않으면 E0046입니다. `download_mb`는 기본 구현(`None`)이 맞는 답이므로 쓰지 않아도 됩니다. 열거형판은 배리언트를 더하면 `title`, `study_minutes`, `download_mb`의 `match`가 E0004로 하나씩 알려 줍니다. 열거형판에서 고칠 곳이 가장 많지만 빠뜨릴 수 없고, `Any`판은 고칠 곳을 빠뜨려도 알 수 없습니다.

</details>

### 연습 2. `TypeId` 없이 꼬리표 붙이기

4.3절의 `label` 함수를 `TypeId`와 `'static` 없이 다시 써 보세요. 퀴즈에는 `[퀴즈]`, 나중에 더할 시험에는 `[시험]` 꼬리표가 붙어야 하고, 꼬리표가 없는 자료는 제목만 나와야 합니다. 빌린 문자열로 제목을 들고 있는 `Preview<'a>`도 받을 수 있는지 확인해 보세요.

<details>
<summary>힌트</summary>

`Lesson`에 `fn badge(&self) -> Option<&'static str> { None }`을 더하고 `Quiz`에서 `Some("퀴즈")`를 돌려주도록 재정의합니다. `label`은 `match item.badge() { Some(b) => format!("[{b}] {}", item.title()), None => item.title().to_string() }`이면 되고, `T`에 `'static` 조건이 없으므로 `Preview<'a>`도 받습니다. 시험 타입을 더할 때 `label`은 고치지 않습니다.

</details>

### 연습 3. 사용자가 고른 열로 내보내기

예제 05에 `fn select_columns(names: &[&str]) -> Result<Vec<Column>, String>`을 더해 보세요. `COLUMNS`에서 머리글이 같은 열을 골라 순서대로 돌려주고, 모르는 이름이 있으면 그 이름을 담은 `Err`를 돌려줍니다. `["learner", "progres"]`를 넘기면 어떤 에러가 나야 할까요?

<details>
<summary>힌트</summary>

이름마다 `COLUMNS.iter().find(|(header, _)| header == name)`으로 찾고, `ok_or_else(|| format!("모르는 열: {name}"))`로 `Result`를 만든 뒤 `collect::<Result<Vec<_>, _>>()`로 모읍니다([ch9](../ch9-iterator-transforms/README.md)). `Column`은 `&'static str`과 함수 포인터의 튜플이라 `Copy`이므로 `*column`으로 복사해 담을 수 있습니다. `"progres"`에서 `Err("모르는 열: progres")`가 나와야 합니다. 1번 구역과 달리 오타가 `?`로 조용히 바뀌지 않고, 문자열이 필드로 바뀌는 곳은 이 함수 하나뿐입니다.

</details>

### 연습 4. 무엇으로 풀까

다음 상황마다 `type_name`, `Any` 다운캐스트, 트레이트 메서드, 열거형, derive 중 무엇을 쓸지 고르고 이유를 적어 보세요.

1. 제네릭 캐시 함수가 실패했을 때, 어떤 타입의 값을 넣으려다 실패했는지 로그에 남깁니다.
2. 플러그인들이 요청마다 자기만의 데이터를 붙여 두고, 나중에 같은 플러그인이 자기 데이터를 타입으로 찾아 꺼냅니다.
3. 학습 자료마다 "자막 제공 여부"를 화면에 표시합니다. 자료 타입은 다른 팀도 계속 더합니다.
4. 수강 기록 구조체 전체를 오류 보고에 첨부합니다.
5. 동영상 화질은 360p, 720p, 1080p 세 가지뿐이고 화질마다 비트레이트가 다릅니다.

<details>
<summary>힌트</summary>

1번은 사람이 읽고 끝나는 로그이므로 `type_name::<T>()`가 맞습니다. 2번은 담는 쪽이 들어올 타입을 모르는 타입 맵이라 `Any`가 정당한 경우입니다(Part 1 ch14 2.8절). 3번은 열린 집합이고 모든 자료가 답해야 하는 질문이므로 트레이트의 필수 메서드(`fn has_subtitles(&self) -> bool`)입니다. 대부분이 제공하지 않는다면 기본 구현 `false`를 둘 수도 있지만, 빠뜨려도 알 수 없게 된다는 점을 따져 봅니다. 4번은 `#[derive(Debug)]`와 `{:?}`입니다. 5번은 닫힌 집합이므로 열거형과 `match`입니다.

</details>

## 6. 참고 자료

- [Effective Rust, Item 19: Avoid reflection](https://effective-rust.com/reflection.html): 이 챕터가 다루는 지침의 출처입니다. 리플렉션과 RTTI를 소개하고, `type_name`, `TypeId`, `Any`의 한계와 트레이트 객체가 상속이 아니라는 점, 대안(트레이트, 마커 트레이트, derive 매크로)을 차례로 설명합니다.
- [`std::any` 모듈 문서](https://doc.rust-lang.org/std/any/index.html): `&dyn Any`로는 트레이트 구현 여부를 확인할 수 없다는 설명과 스마트 포인터에서 `type_id()`를 부를 때의 주의가 있습니다. 개별 항목: [`type_name`](https://doc.rust-lang.org/std/any/fn.type_name.html), [`type_name_of_val`](https://doc.rust-lang.org/std/any/fn.type_name_of_val.html), [`TypeId`](https://doc.rust-lang.org/std/any/struct.TypeId.html), [`Any`](https://doc.rust-lang.org/std/any/trait.Any.html)
- [`std::fmt::Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html): derive한 `Debug`가 무엇을 출력하는지와 그 형식이 안정적이지 않다는 설명입니다.
- [Announcing Rust 1.86.0](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/): 트레이트 객체 업캐스트가 안정화된 릴리스의 발표입니다. `dyn Any`로 올려 다운캐스트하는 쓰임도 소개합니다.
- [rust-lang/rust#65991](https://github.com/rust-lang/rust/issues/65991)(트레이트 업캐스트), [rust-lang/rust#41875](https://github.com/rust-lang/rust/issues/41875)(`'static`이 아닌 타입의 `TypeId`): 책이 가리키는 이슈입니다.
- 본편: [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md)(`Any`, `TypeId`, `type_name`, 다운캐스트, 타입 맵), [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md)(트레이트 객체, 열거형과의 비교), [Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md)(슈퍼트레이트, derive 매크로), [Part 1 ch8](../../part1-standard-lib/ch8-errors-panic/README.md)(에러 다운캐스트)
- 부록: [ch2](../ch2-express-common-behavior/README.md)(마커 트레이트), [ch4](../ch4-idiomatic-errors/README.md)(에러의 종류를 지우는 트레이트 객체), [ch5](../ch5-type-conversions/README.md)(강제 변환으로서의 업캐스트), [ch12](../ch12-generics-vs-trait-objects/README.md)(조합 트레이트, 업캐스트, 열린 집합과 닫힌 집합), [ch13](../ch13-default-implementations/README.md)(기본 구현)
