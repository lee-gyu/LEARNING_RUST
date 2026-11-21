# ch6. 클래스도 상속도 없이

Rust에는 클래스(class)도 상속(inheritance)도 없습니다. Java, C#, C++, Kotlin, Python으로 객체 지향 설계를 해 온 사람은 곧바로 묻습니다. 데이터와 메서드는 어디에 묶는가, 생성자는 어디 있는가, 부모 클래스에 두던 공통 코드는 어디로 가는가, 오버로딩과 기본 인자는 왜 없는가. 이 챕터는 구조체·트레이트·제네릭의 문법을 다시 가르치지 않습니다(Part 0 ch5·ch10·ch17의 일입니다). 대신 독자가 쓰던 언어의 설계 도구를 Rust의 무엇으로 옮기면 되는지, 그 대가는 무엇인지에 답합니다.

- 패키지: `faq-ch6-no-classes-inheritance`
- 확인한 툴체인: rustc 1.100.0, cargo 1.100.0 (2024 에디션), macOS(aarch64)
- 자료 확인일: 2026-11-21

## 이 챕터가 답하는 질문

| 번호 | 질문 | 짧은 답 |
| --- | --- | --- |
| [Q6.1](#q61-rust는-객체-지향-언어인가요) | Rust는 객체 지향 언어인가요? | 데이터와 동작을 묶고 캡슐화한다는 뜻이면 그렇고, 상속이 있어야 한다는 뜻이면 아닙니다. |
| [Q6.2](#q62-클래스가-없으면-데이터와-메서드를-어떻게-묶나요) | 클래스가 없으면 데이터와 메서드를 어떻게 묶나요? | 데이터는 `struct`·`enum`으로, 메서드는 그 타입의 `impl` 블록으로 따로 적습니다. |
| [Q6.3](#q63-생성자가-없나요) | 생성자가 없나요? | 없습니다. 모든 필드를 채운 구조체 리터럴로 만들고, `new`는 그것을 감싼 함수에 붙이는 관례적 이름입니다. |
| [Q6.4](#q64-상속이-없는데-코드-재사용과-다형성은-어떻게-하나요) | 상속이 없는데 코드 재사용과 다형성은 어떻게 하나요? | 재사용은 트레이트의 기본 메서드와 조합(composition)으로, 다형성은 제네릭과 트레이트 객체로 합니다. |
| [Q6.5](#q65-트레이트는-javac의-인터페이스-go의-인터페이스와-무엇이-다른가요) | 트레이트는 Java·C#의 인터페이스, Go의 인터페이스와 무엇이 다른가요? | 구현을 `impl`로 따로 밝히고, 그 `impl`을 남의 타입에도 쓸 수 있으며, 제네릭 바운드로 쓰면 정적 디스패치가 됩니다. |
| [Q6.6](#q66-제네릭은-java타입-소거-c템플릿-go와-어떻게-다른가요) | 제네릭은 Java(타입 소거), C++(템플릿), Go와 어떻게 다른가요? | 쓰인 타입마다 코드를 따로 만들고(단형성화), 본문에서 타입 매개변수로 하는 일은 모두 트레이트 바운드로 밝혀야 합니다. |
| [Q6.7](#q67-함수-오버로딩-기본-인자-가변-인자가-없나요-대신-무엇을-쓰나요) | 함수 오버로딩, 기본 인자, 가변 인자가 없나요? 대신 무엇을 쓰나요? | 없습니다. 다른 이름의 함수와 트레이트, `Default`와 빌더, 슬라이스와 매크로를 씁니다. |
| [Q6.8](#q68-instanceof와-다운캐스트-추상-클래스-계층은-어떻게-옮기나요) | `instanceof`와 다운캐스트, 추상 클래스 계층은 어떻게 옮기나요? | 닫힌 계층은 열거형과 `match`, 열린 계층은 트레이트 객체로 옮깁니다. 다운캐스트는 마지막 수단입니다. |
| [Q6.9](#q69-리플렉션-어노테이션-데코레이터에-해당하는-것은-무엇인가요) | 리플렉션, 어노테이션, 데코레이터에 해당하는 것은 무엇인가요? | 실행 중 리플렉션은 없고, 컴파일 시간에 코드를 만드는 매크로(`#[derive]`, 속성 매크로)가 그 자리를 맡습니다. |
| [Q6.10](#q610-private-protected는-어떻게-되나요) | `private`, `protected`는 어떻게 되나요? | 기본이 비공개이고 단위는 모듈입니다. `protected`는 없고 `pub(crate)`·`pub(super)`로 범위를 좁힙니다. |

다음 챕터의 내용을 안다고 보고 진행합니다.

- [Part 0 ch5](../../part0-getting-started/ch5-structs/README.md): 구조체, 메서드와 `self`, 연관 함수
- [Part 0 ch7](../../part0-getting-started/ch7-modules/README.md): 모듈과 `pub`, 기본 비공개
- [Part 0 ch10](../../part0-getting-started/ch10-generics-traits-lifetimes/README.md): 제네릭과 단형성화(2.4절), 트레이트와 기본 구현, 고아 규칙(2.7절), 트레이트 바운드
- [Part 0 ch17](../../part0-getting-started/ch17-trait-objects-oop/README.md): Rust와 객체 지향(2.1·2.3절), 트레이트 객체와 동적 디스패치
- [ch5](../ch5-no-null-no-exceptions/README.md) Q5.3(실패는 `Result`로 알린다)

## 1. 학습 목표

이 챕터를 마치면 다음을 할 수 있습니다.

1. Java식 클래스 계층(필드 상속, `super` 호출, 템플릿 메서드)을 구조체 조합, 위임, 트레이트의 기본 메서드로 옮길 수 있다.
2. 트레이트를 Java·Go의 인터페이스와, Rust 제네릭을 Java의 타입 소거·C++ 템플릿·Go 제네릭과 비교해 차이를 설명할 수 있다.
3. 생성자 오버로딩, 기본 인자, 가변 인자, 인자 타입에 따른 오버로딩을 Rust의 관용구로 바꿔 쓸 수 있다.
4. 타입 계층을 열거형과 트레이트 객체 중 무엇으로 옮길지 고르고, 리플렉션·어노테이션과 접근 제어자가 Rust에서 무엇에 해당하는지 설명할 수 있다.

## 2. 핵심 개념

질문마다 **짧은 답 → 왜 그런가 → 다른 언어에서는 → 오해와 한계 → 더 읽기** 순서로 답합니다.

### Q6.1 Rust는 객체 지향 언어인가요?

**짧은 답** 무엇을 객체 지향이라 부르느냐에 따라 다릅니다. "객체는 데이터와 그 데이터를 다루는 동작을 묶은 것"이라는 뜻이면 Rust도 객체 지향입니다. 구조체와 열거형에 `impl`로 메서드를 붙이고, 비공개 필드로 캡슐화합니다. 하지만 클래스와 상속은 없으므로, 상속이 있어야 객체 지향이라고 본다면 Rust는 객체 지향 언어가 아닙니다. The Book 18.1절도 같은 결론을 내립니다.

**왜 그런가** The Book은 상속을 쓰는 이유를 둘로 나눕니다. 코드를 다시 쓰지 않고 물려받는 재사용, 그리고 자식 타입을 부모 타입 자리에 쓰는 다형성(polymorphism)입니다. Rust는 상속 없이 이 둘을 따로 제공합니다(Q6.4). The Book은 그 이유로 상속이 필요 이상의 코드를 공유하게 만들기 쉽다는 점을 듭니다.

**다른 언어에서는** Go FAQ도 "Go는 객체 지향 언어인가"에 "그렇기도 하고 아니기도 하다"고 답합니다. 타입과 메서드가 있고 객체 지향 스타일로 쓸 수 있지만 타입 계층은 없고, 대신 인터페이스와 임베딩(embedding)을 둔다는 설명입니다. Rust는 이 점에서 Java·C#보다 Go에 가깝습니다. 다만 Go와 달리 인터페이스(트레이트) 구현을 명시하고(Q6.5) 임베딩이 없습니다(Q6.4).

**오해와 한계** 상속이 없다고 객체 지향 설계 패턴을 못 쓰는 것은 아닙니다. Part 0 ch17은 상태 패턴을 트레이트 객체로 옮깁니다. 다만 기반 클래스가 상태와 동작을 함께 물려주는 깊은 계층을 전제로 한 설계는 그대로 옮길 수 없고, 상태는 조합으로, 동작은 트레이트로 나눠 다시 설계해야 합니다.

**더 읽기** Part 0 ch17 2.1·2.3·2.11절

### Q6.2 클래스가 없으면 데이터와 메서드를 어떻게 묶나요?

**짧은 답** 데이터는 `struct`(또는 `enum`)로 정의하고, 메서드는 그 타입의 `impl` 블록에 따로 적습니다. 한 타입에 `impl` 블록을 여러 개 둘 수 있고, 트레이트 구현도 `impl 트레이트 for 타입`이라는 별도의 블록입니다. 클래스 하나에 필드·메서드·인터페이스 구현을 모두 적는 대신, 데이터 정의와 동작을 나눠 적는 구조입니다.

**왜 그런가** [01_inheritance_to_composition](examples/01_inheritance_to_composition.rs)의 `Employee`는 필드를 정의한 `struct Employee`, 생성 함수를 둔 `impl Employee`, 트레이트를 구현한 `impl Payable for Employee` 세 블록으로 나뉩니다. 메서드는 첫 매개변수 `self`의 모양(`&self`, `&mut self`, `self`)으로 값을 빌릴지, 고칠지, 가져갈지를 밝히며(Part 0 ch5 2.9절), 메서드 안에서 필드는 언제나 `self.name`처럼 씁니다. 열거형에도 같은 방식으로 메서드를 붙입니다([Part 0 ch6](../../part0-getting-started/ch6-enums-pattern-matching/README.md)).

**다른 언어에서는** Go가 가장 비슷합니다. 구조체를 정의하고 메서드는 수신자(receiver)를 받는 함수로 따로 선언하며, Go 명세는 수신자의 기본 타입이 메서드와 같은 패키지에 선언돼 있어야 한다고 정합니다. Rust의 고유 메서드(inherent method, 트레이트가 아니라 타입 자신의 `impl`에 둔 메서드)도 비슷한 제약이 있어서, 타입을 정의한 크레이트 밖에서는 `impl String { … }`처럼 쓸 수 없습니다(E0116). 남의 타입에 메서드를 더하는 일은 트레이트로 합니다(Q6.5).

**오해와 한계** Java의 `this`처럼 숨은 수신자가 없어서 `self`와 `self.`을 매번 적어야 합니다. 대신 메서드가 값을 고치는지(`&mut self`), 소비하는지(`self`)가 시그니처에 드러납니다.

**더 읽기** Part 0 ch5 2.9~2.12절

### Q6.3 생성자가 없나요?

**짧은 답** 언어가 정한 생성자는 없습니다. 값은 구조체 리터럴(`Timer { label, duration }`)로 만들고, 필드를 하나라도 빠뜨리면 컴파일되지 않습니다(E0063). `new`는 이 리터럴을 감싼 연관 함수에 붙이는 관례적인 이름일 뿐이며, Rust API 가이드라인도 생성자를 "관례"로 설명합니다(C-CTOR). 이름이 자유로우므로 만드는 방법마다 `new`, `with_capacity`, `from_minutes`처럼 다른 이름을 붙입니다.

**왜 그런가** 필드가 비공개면 모듈 밖에서는 구조체 리터럴을 쓸 수 없으므로, `new` 같은 공개 함수가 값을 만드는 유일한 입구가 됩니다. 불변식은 그 입구에서 한 번 검사하면 됩니다([Part 0 ch9](../../part0-getting-started/ch9-error-handling/README.md) 2.12절의 `Guess`, Part 0 ch17 2.2절). 또 Rust의 값은 모든 필드가 채워진 뒤에야 생기므로, 생성자가 도는 동안 반쯤 만들어진 객체가 바깥에 보이는 일이 없습니다. 실패할 수 있는 생성은 예외 대신 `Result<Self, E>`를 돌려줍니다(ch5 Q5.3).

[03_overloading_alternatives](examples/03_overloading_alternatives.rs)의 1번은 생성자 오버로딩 대신 이름이 다른 두 연관 함수를 씁니다.

```text
1. 생성자 오버로딩 대신 이름이 다른 연관 함수
  차: 180s
  회의: 1800s
```

**다른 언어에서는** Java는 새 객체의 인스턴스 필드를 먼저 기본값(참조 타입은 `null`, 수는 0)으로 채운 뒤 생성자를 실행합니다(Java 언어 명세 12.5, 기본값은 4.12.5). Go에는 생성자가 없고, Effective Go는 제로 값으로 부족할 때 `NewFile`처럼 초기화 함수를 두고 복합 리터럴(composite literal)로 값을 만드는 예를 보입니다. Rust의 `new`는 Go의 `NewXxx` 관례와 같은 자리에 있고, 차이는 제로 값이 없다는 것입니다. 기본값이 필요하면 `Default` 트레이트로 명시합니다.

**오해와 한계** 생성자 체이닝이나 기본 인자가 없어서 필드가 많은 타입은 만드는 코드가 길어집니다. 그때는 `Default`와 구조체 갱신 문법(struct update syntax)이나 빌더를 씁니다([부록 A ch7](../../appd-a/ch7-builders/README.md)).

**더 읽기** Part 0 ch5 2.11절 / [Part 1 ch6](../../part1-standard-lib/ch6-core-traits/README.md)(`Default`) / 부록 A ch7

### Q6.4 상속이 없는데 코드 재사용과 다형성은 어떻게 하나요?

**짧은 답** The Book이 꼽는 상속의 두 쓰임새로 나눠 답합니다. 코드 재사용은 트레이트의 기본 메서드와 조합(다른 타입의 값을 필드로 품기)으로, 다형성은 제네릭(정적 디스패치)이나 트레이트 객체(동적 디스패치)로 합니다. 부모의 필드를 물려받는 방법은 없으므로, 공통 상태는 조합으로 품고 필요한 메서드를 직접 넘겨줍니다(위임, delegation).

**왜 그런가** 01 예제는 Java의 `Employee`/`Manager` 계층을 옮깁니다.

```text
1. 기본 메서드와 위임
  김하나: 월 300만 원
  이두리: 월 450만 원
```

| Java | Rust(01 예제) |
| --- | --- |
| 부모 클래스의 메서드 `paySlip()` | 트레이트 `Payable`의 기본 메서드 `pay_slip` |
| `class Manager extends Employee` | `struct Manager { base: Employee, bonus: u32 }` |
| 물려받은 필드 `name` | `self.base.name()`으로 위임 |
| `super.monthlyPay()` | `self.base.monthly_pay()` |
| `List<Employee>`에 `Manager`도 담기 | `[&dyn Payable; 2]`(트레이트 객체, Part 0 ch17) |

트레이트는 필드를 가질 수 없으므로, 기본 메서드는 필요한 값을 `name()`, `monthly_pay()` 같은 필수 메서드로 얻습니다. 부모 클래스의 필드를 직접 읽던 코드가 메서드 호출로 바뀌는 셈입니다. 기본 메서드와 그 재정의 자체는 Part 0 ch10 예제 03에서 봤습니다.

**다른 언어에서는** Go는 구조체에 다른 타입을 이름 없이 넣는 임베딩을 지원하고, Go 명세는 그 안쪽 타입의 필드와 메서드가 바깥 구조체로 "승격(promoted)"된다고 정합니다. Go FAQ는 이것을 서브클래싱과 비슷하지만 같지는 않은 것으로 설명합니다. Rust에는 임베딩이 없어서, 01의 `Manager`처럼 품은 값의 메서드를 손으로 넘겨줘야 합니다.

**오해와 한계** 조합은 하위 타입 관계를 만들지 않습니다. `Manager`가 `Employee`를 품어도 `&Employee`를 받는 함수에 `&Manager`를 넘길 수 없습니다(4.1절). 여러 타입을 받아야 하는 함수는 트레이트로 받습니다. 또 재정의한 메서드 안에서 기본 구현을 불러 쓸 수 없어서, Java의 `super.paySlip()`처럼 부모의 구현에 덧붙이는 코드는 쓸 수 없습니다(Part 0 ch17 2.3절). 위임할 메서드가 많으면 같은 모양의 코드가 늘어나는 것도 비용입니다. `Deref`를 구현해 안쪽 값의 메서드가 저절로 보이게 흉내 낼 수 있지만, Rust API 가이드라인은 `Deref`를 스마트 포인터에만 구현하라고 합니다(C-DEREF). 이 트레이트는 컴파일러가 여러 곳에서 암묵적으로 쓰고 메서드 찾기 규칙과 얽혀 있어서, 그 규칙이 스마트 포인터에 맞춰 설계됐다는 설명입니다.

**더 읽기** Part 0 ch10 2.5절 / Part 0 ch17 2.3·2.6절 / [부록 A ch12](../../appd-a/ch12-generics-vs-trait-objects/README.md)(제네릭과 트레이트 객체 고르기)

### Q6.5 트레이트는 Java·C#의 인터페이스, Go의 인터페이스와 무엇이 다른가요?

**짧은 답** "이런 메서드를 가진 타입"이라는 약속이라는 점은 같습니다. 다른 점은 셋입니다. 구현했다는 사실을 `impl 트레이트 for 타입`으로 따로 밝혀야 하고(Go처럼 메서드만 갖추면 저절로 구현되지 않음), 그 `impl`을 타입 정의와 떨어진 곳에 쓸 수 있어 남이 만든 타입에도 내 트레이트를 구현할 수 있으며(고아 규칙 안에서), 제네릭 바운드로 쓰면 정적 디스패치가 됩니다.

**왜 그런가**

| | Java 인터페이스 | Go 인터페이스 | Rust 트레이트 |
| --- | --- | --- | --- |
| 구현했다는 표시 | 클래스 선언의 `implements` 절 | 없음. 메서드를 모두 가지면 구현(구조적 타이핑) | `impl 트레이트 for 타입` 블록 |
| 이미 있는 타입에 붙이기 | 그 클래스의 선언을 고쳐야 함 | 메서드는 타입과 같은 패키지에서만 정의 | 트레이트나 타입 중 하나가 내 크레이트 것이면 가능(고아 규칙, Part 0 ch10 2.7절) |
| 기본 구현 | `default` 메서드(Java 언어 명세 9.4) | 없음 | 기본 메서드 |
| 쓰는 방식 | 인터페이스 타입의 변수, 제네릭의 상한 | 인터페이스 타입의 값, 제네릭의 제약 | 제네릭 바운드(정적 디스패치) 또는 `dyn 트레이트`(동적 디스패치) |

- 세 언어 모두 제네릭의 제약으로도 쓰고, 차이는 그때 코드를 만드는 방식입니다(Q6.6).
- Go FAQ는 Go 타입이 인터페이스의 메서드를 구현하는 것만으로 그 인터페이스를 구현하며, 이것이 일종의 구조적 타이핑(structural typing)이라고 설명합니다. 같은 FAQ는 Rust에서는 트레이트 바운드와 타입의 관계를 트레이트를 정의한 크레이트나 타입을 정의한 크레이트에서 명시해야 한다고 대비합니다.
- 그래서 Go식으로 생각해 같은 이름의 메서드만 만들면 E0277이 납니다. 타입 자신의 메서드와 트레이트의 메서드는 이름만 같은 별개의 함수라는 것은 Part 0 ch10 4.2절이 설명합니다.
- C#의 인터페이스도 멤버의 기본 구현을 둘 수 있습니다(C# 문서).
- Kotlin의 확장 함수(extension function)처럼 남의 타입에 메서드를 더하는 일은, Rust에서는 트레이트를 정의해 그 타입에 구현하는 방식으로 합니다. Part 0 ch10 예제 03이 내 트레이트 `Summary`를 표준 라이브러리의 `Vec`에 구현합니다. Kotlin 문서는 확장 함수가 정적으로 결정된다고 설명합니다.

**오해와 한계** 명시적인 `impl`은 "우연히 메서드 이름이 같아서 구현된 것으로 취급되는" 일을 막는 대신, 이미 가진 메서드만으로 저절로 만족되는 Go의 편리함은 없어서 타입마다 `impl`을 써야 합니다. 고아 규칙 때문에 남의 트레이트를 남의 타입에 구현할 수 없어 뉴타입으로 감싸야 할 때가 있고([Part 0 ch19](../../part0-getting-started/ch19-advanced-features/README.md) 2.12절), 트레이트의 메서드는 그 트레이트를 `use`로 가져와야 부를 수 있습니다(Part 0 ch10 2.6절).

**더 읽기** Part 0 ch10 2.5~2.7·2.11절 / Part 0 ch17 2.9절 / [부록 A ch2](../../appd-a/ch2-express-common-behavior/README.md)

### Q6.6 제네릭은 Java(타입 소거), C++(템플릿), Go와 어떻게 다른가요?

**짧은 답** Rust의 제네릭은 쓰인 구체 타입마다 코드를 따로 만듭니다(단형성화, monomorphization). 그래서 Java처럼 타입 인자가 지워지지 않고(타입 소거, type erasure), 제네릭 때문에 실행이 느려지지도 않습니다. C++ 템플릿과 다른 점은 본문에서 `T`로 하는 일(비교, 출력, 복제 등)을 모두 트레이트 바운드로 밝혀야 하고, 본문은 정의할 때 그 바운드만으로 검사된다는 것입니다. 바운드 없는 `T`로는 옮기기·빌리기·버리기 정도만 할 수 있습니다. Go와 다른 점은 타입이 바운드를 만족한다는 것을 `impl`로 밝혀야 한다는 것입니다.

**왜 그런가** The Book 10.1절은 단형성화를 컴파일할 때 제네릭 코드에 실제로 쓰인 구체 타입을 채워 넣어 특정 코드로 바꾸는 과정으로 설명하고, 그래서 제네릭을 써도 구체 타입으로 쓴 코드보다 느려지지 않는다고 합니다. 각 사본 안에서 `T`는 이미 구체 타입이므로 [02_generics_not_erased](examples/02_generics_not_erased.rs)처럼 Java에서는 막힌 일을 할 수 있습니다.

```text
1. 타입 인자로 새 값 만들기
  u8: 0 (1바이트)
  u64: 0 (8바이트)
  String: "" (24바이트)
2. 타입 인자만 다른 타입에 다른 구현
  바이트 4개
  문자열 2개: 가, 나
```

Java 튜토리얼의 "Restrictions on Generics"는 타입 매개변수로 인스턴스를 만들 수 없고(`new T()`), 기본형을 타입 인자로 쓸 수 없으며, 소거하면 같은 타입이 되는 매개변수로 메서드를 오버로딩할 수 없다고 적습니다. 02의 `T::default()`, `fresh::<u8>()`, `Vec<u8>`과 `Vec<String>`에 따로 둔 `impl`이 차례로 그 세 가지에 대응합니다.

**다른 언어에서는** Go FAQ의 "다른 언어의 제네릭과 비교" 항목이 Go·Java·C++·Rust를 한 번에 비교합니다. 아래 표는 그 항목과 같은 FAQ의 "Go의 제네릭은 어떻게 구현되나" 항목, Java 튜토리얼(Bounded Type Parameters, Restrictions on Generics)을 근거로 했습니다.

| | Java | C++ 템플릿 | Go | Rust |
| --- | --- | --- | --- | --- |
| 타입 인자의 제약 | 선택(`extends`로 상한 지정) | 전통적으로 없음. C++20 concepts로 선택 | 반드시(인터페이스 타입으로 쓴 제약) | 선택. 다만 본문에서 쓰는 능력은 모두 트레이트 바운드로 적어야 함(Part 0 ch10의 E0369) |
| 제약을 만족시키는 법 | 그 클래스를 상속하거나 인터페이스를 구현 | 템플릿 본문이 그 타입으로 컴파일되면 됨 | 암묵적으로(제약이 허용하는 타입 집합에 들면) | `impl`로 명시 |
| 컴파일 결과 | 타입 인자를 지움(타입 소거). 실행 중에 어떤 타입 인자로 쓰였는지 확인할 수 없음 | 쓰인 곳마다 템플릿을 따로 컴파일 | 컴파일러가 고름. 표준 컴파일러는 크기와 포인터 위치 같은 "모양"이 같은 타입 인자끼리 하나를 공유 | 쓰인 타입마다 따로(단형성화) |

**오해와 한계** 단형성화의 대가는 쓰인 타입마다 사본이 생겨 컴파일 시간과 실행 파일 크기가 늘 수 있다는 것입니다. 그것이 문제일 때 제네릭 부분을 얇게 줄이거나 트레이트 객체를 쓰는 방법은 부록 A ch12에 있습니다. 또 C++ 템플릿처럼 "일단 써 보고 쓰인 곳에서 컴파일되면 통과"하는 방식이 아니어서, 본문에서 쓰는 능력은 모두 바운드로 적어야 합니다(Part 0 ch10의 E0369).

**더 읽기** Part 0 ch10 2.1~2.4·2.9절 / 부록 A ch12 / [ch14](../ch14-performance/README.md)

### Q6.7 함수 오버로딩, 기본 인자, 가변 인자가 없나요? 대신 무엇을 쓰나요?

**짧은 답** 셋 다 없습니다. 오버로딩 대신 이름이 다른 함수를 두거나, 받을 수 있는 타입들이 구현하는 트레이트를 인자 타입으로 씁니다(`impl Into<String>` 같은 제네릭 인자). 기본 인자 대신 `Default`와 구조체 갱신 문법, 빌더를 쓰고, 가변 인자 대신 슬라이스 `&[T]`를 받거나 매크로(`vec!`, `println!`)를 씁니다.

**왜 그런가** 03 예제의 나머지 두 구역입니다.

```text
2. 가변 인자 대신 슬라이스
  합계: 1980s
  빈 목록: 0ns
3. 인자 타입에 따른 오버로딩 대신 트레이트
  90초 기다림
  2초 기다림
```

- 1번(Q6.3)의 `Timer::new`는 `label: impl Into<String>`을 받아 `&str`도 `String`도 받습니다. `String`이 `From<&str>`을 구현하므로 `&str`에도 `Into<String>`이 있기 때문입니다. 같은 모양의 생성자는 Part 1 ch6 예제 01에서 봤습니다.
- 3번의 `wait`는 `u64`와 `Duration`이 모두 구현한 트레이트 `IntoSecs`를 인자 타입으로 받습니다. `wait(90)`의 `90`은 `IntoSecs`를 구현한 정수 타입이 `u64` 하나뿐이라 `u64`로 정해집니다. `u32`에도 구현하면 리터럴이 기본 정수 타입 `i32`가 되어 E0277이 나므로, 오버로딩을 트레이트로 옮길 때 주의할 점입니다. 오버로딩이라면 컴파일러가 인자 타입을 보고 함수를 고르는 일을, 여기서는 트레이트 구현이 맡습니다. 어떤 타입을 받는지가 트레이트 구현 목록으로 드러납니다.
- 연산자는 예외입니다. `+`, `==` 같은 연산자는 `Add`, `PartialEq` 같은 트레이트를 구현해 타입마다 다르게 정의합니다(Part 0 ch19 2.9절).

**다른 언어에서는** Java는 이름이 같고 시그니처가 다른 메서드를 오버로딩으로 허용합니다(Java 언어 명세 8.4.9). Go FAQ는 오버로딩이 가끔 쓸모 있지만 실제로는 헷갈리고 깨지기 쉬웠다는 경험을 들어 Go에 넣지 않았다고 설명합니다. Go의 가변 인자 `...T`는 함수 안에서 슬라이스 `[]T`이므로(Go 명세), Rust에서 슬라이스를 받는 것과 같은 구조입니다. Kotlin 문서는 기본값을 가진 매개변수가 오버로딩을 여러 개 만들 필요를 줄여 준다고 설명하는데, Rust에서 그 자리는 `Default`와 빌더가 맡습니다.

**오해와 한계** 만드는 방법마다 이름을 지어야 하고, 이름 있는 인자(named argument)도 없어서 인자가 많은 함수는 설정 구조체나 빌더로 바꾸게 됩니다(부록 A ch7). `impl Into<String>` 같은 제네릭 인자는 편하지만 함수가 제네릭이 되므로 쓰인 타입마다 사본이 생기고(Q6.6), 남발하면 시그니처를 읽기 어려워집니다. 같은 타입의 `impl`에 같은 이름의 연관 함수를 두 개 만들면 4.2절의 에러(E0592)가, 한 모듈에 같은 이름의 함수를 두 개 만들면 E0428이 납니다.

**더 읽기** Part 0 ch19 2.9·2.19절 / Part 1 ch6(`From`·`Into`, `Default`) / 부록 A ch7

### Q6.8 `instanceof`와 다운캐스트, 추상 클래스 계층은 어떻게 옮기나요?

**짧은 답** 종류가 정해진 닫힌 계층이면 열거형과 `match`로 옮깁니다. `instanceof`로 갈라 처리하던 코드는 `match`가 되고, 빠뜨린 종류는 컴파일러가 알려 줍니다. 다른 크레이트가 종류를 더할 수 있는 열린 계층이면 트레이트 객체로 옮기고, 종류별 처리를 트레이트 메서드에 둡니다. 다운캐스트는 `std::any::Any`로 할 수 있지만 `'static` 타입에만 되고 마지막 수단입니다. 추상 클래스는 기본 메서드가 있는 트레이트와 조합으로 옮깁니다(Q6.4).

**왜 그런가** 열거형은 배리언트마다 클래스를 만들지 않고 값 하나에 데이터를 담으며, `match`가 모든 배리언트를 다뤘는지 검사합니다(Part 0 ch6). 반대로 열거형은 배리언트를 더하려면 그 열거형을 고쳐야 하므로, 바깥에서 종류를 더해야 하면 트레이트 객체가 맞습니다. 열거형·제네릭·트레이트 객체 중 무엇을 고를지는 Part 0 ch17 2.10절에 있고, 같은 문제를 `Any` 다운캐스트 사슬·트레이트 메서드·열거형으로 각각 풀어 비교한 예제는 [부록 A ch19](../../appd-a/ch19-avoid-reflection/README.md) 예제 02입니다.

**다른 언어에서는** 요즘 Java와 Kotlin에도 같은 도구가 있습니다. Java 언어 명세는 `sealed` 인터페이스의 허용된 하위 타입을 모두 다룬 `switch`를 빠짐없는(exhaustive) 것으로 인정합니다(14.11.1.1). Kotlin 문서도 `sealed` 클래스와 `when`을 함께 쓰면 모든 하위 클래스를 다룰 수 있다고 설명합니다. Rust의 열거형과 `match`는 이 관용구에 해당합니다.

**오해와 한계** 열거형의 배리언트는 그 자체로는 타입이 아니어서, "원 모양만 받는 함수"처럼 배리언트 하나를 매개변수 타입으로 쓸 수 없습니다. 그럴 때는 배리언트가 구조체를 담게 하고(`Shape::Circle(Circle)`) 그 구조체를 받습니다. `Any`로 다운캐스트하는 코드는 종류가 늘 때마다 고칠 곳이 흩어지므로 부록 A ch19는 트레이트 메서드를 먼저 권합니다.

**더 읽기** Part 0 ch6 / Part 0 ch17 2.10절 / [Part 1 ch14](../../part1-standard-lib/ch14-ffi-any-misc/README.md)(`Any`) / 부록 A ch19

### Q6.9 리플렉션, 어노테이션, 데코레이터에 해당하는 것은 무엇인가요?

**짧은 답** 실행 중에 타입의 필드와 메서드를 살피는 리플렉션(reflection)은 없습니다. 그런 정보가 필요한 일(직렬화, `equals`·`toString` 만들기 등)은 컴파일 시간에 코드를 만들어 내는 매크로가 맡습니다. 어노테이션 자리에 오는 `#[derive(...)]`와 속성 매크로(attribute macro)는 컴파일러가 실행해 코드를 생성하는 절차적 매크로(procedural macro)입니다. 실행 중에 알 수 있는 것은 `std::any`의 `TypeId`·`type_name` 정도이고 쓰임새가 좁습니다.

**왜 그런가** 코드를 컴파일 시간에 만들면 실행 중에 타입 정보를 들고 다닐 필요가 없고, 만든 코드도 보통 코드와 똑같이 타입 검사를 받습니다. `#[derive(Debug, Clone, PartialEq)]`가 그런 예이고(Part 0 ch10 2.8절), 절차적 매크로를 직접 만드는 법은 Part 0 ch19 2.20절에 있습니다.

**다른 언어에서는**

- Java의 레코드(record)는 `equals`, `hashCode`, `toString`을 암묵적으로 선언하고(Java 언어 명세 8.10.3), Kotlin의 데이터 클래스(data class)는 `equals()`/`hashCode()`, `toString()`, `copy()` 등을 컴파일러가 만들며(Kotlin 문서), Python의 `dataclasses`는 `__init__()`·`__repr__()` 같은 메서드를 데코레이터로 더합니다(Python 문서). 셋 다 Rust의 `#[derive]`에 가까운 일입니다.
- Java 언어 명세는 어노테이션을 프로그램 요소에 정보를 붙이는 표시로, 그 자체로는 실행에 영향을 주지 않는다고 정합니다(9.7). 그 정보로 무엇을 할지는 어노테이션을 읽는 도구가 정합니다. Rust의 속성 매크로는 그 처리를 컴파일러 안에서 코드 변환으로 합니다.
- Python 용어집은 데코레이터를 다른 함수를 돌려주는 함수로 설명합니다. Rust에서 함수를 감싸는 일은 보통 클로저를 받는 고차 함수나 감싸는 타입으로 하고, 정의 자체를 바꿔야 할 때 속성 매크로를 씁니다.

**오해와 한계** 매크로는 컴파일 시간을 늘리고, 만들어진 코드가 소스에 보이지 않아 에러를 읽기 어려울 수 있습니다. `type_name`의 결과는 진단용이라 형식이 보장되지 않으므로 그 문자열로 분기하면 안 됩니다(부록 A ch19 예제 01).

**더 읽기** Part 0 ch10 2.8절 / Part 0 ch19 2.18~2.20절 / Part 1 ch14 / 부록 A ch19

### Q6.10 `private`, `protected`는 어떻게 되나요?

**짧은 답** 모든 항목은 기본으로 비공개이고 `pub`을 붙이면 공개됩니다. 다른 점은 단위가 클래스가 아니라 모듈이라는 것입니다. Rust 레퍼런스에 따르면 비공개 항목은 그 항목이 있는 모듈과 그 하위 모듈이 쓸 수 있으므로, 같은 모듈의 다른 타입도 비공개 필드를 읽습니다. 상속이 없으니 `protected`도 없고, 대신 `pub(crate)`(크레이트 안에서만)나 `pub(super)`(부모 모듈까지)로 공개 범위를 좁힙니다.

**왜 그런가** 01의 2번은 `payroll` 모듈 안의 `Manager` 메서드가 `Employee`의 비공개 필드 `annual`을 바로 읽습니다.

```text
2. 같은 모듈이면 다른 타입의 비공개 필드도 읽음
  이두리의 연간 총액: 5400만 원
```

Java라면 `Manager`가 `Employee`의 `private` 필드를 읽을 수 없어 `protected`로 바꿨을 자리입니다. Rust에서는 둘이 같은 모듈에 있다는 것이 접근 권한입니다. 모듈 밖(`main`)에서는 `lead.base`나 `staff.annual`을 쓸 수 없습니다(E0616, Part 0 ch7 4.2절).

**다른 언어에서는** Java의 `private`은 그 선언을 둘러싼 최상위 클래스 안에서만 접근하게 하고, `protected`는 하위 클래스의 코드에서도 접근하게 합니다(Java 언어 명세 6.6). Go는 이름이 대문자로 시작하면 패키지 밖으로 내보내는(exported) 식으로, 공개 단위가 패키지입니다(Go 명세). Rust의 모듈 단위 공개는 Go의 패키지 단위 쪽에 가깝습니다.

**오해와 한계** 공개 단위가 모듈이라서, 한 모듈에 타입을 여럿 두면 서로의 비공개 필드를 건드릴 수 있습니다. 불변식을 지켜야 하는 타입은 하위 모듈로 떼어 내면 그 타입의 메서드만 필드를 고칠 수 있습니다. 구조체는 `pub struct`여도 필드마다 `pub`을 따로 정하고, 열거형은 `pub`이면 배리언트가 모두 공개됩니다(Part 0 ch7 2.6절). 아무것도 붙이지 않았을 때 Java 클래스의 멤버는 패키지 전체에 공개되지만(Java 언어 명세 6.6.1) Rust의 항목은 그 모듈과 하위 모듈에서만 보인다는 점도 다릅니다.

**더 읽기** Part 0 ch7 2.5·2.6절 / Part 0 ch17 2.2절

### 한눈에 보기

| 다른 언어 | Rust | 질문 |
| --- | --- | --- |
| 클래스 | `struct`·`enum`과 `impl` 블록 | Q6.2 |
| 생성자, 생성자 오버로딩 | 구조체 리터럴, `new`·`from_…` 같은 연관 함수 | Q6.3, Q6.7 |
| 필드의 기본값(`null`, 0), Go의 제로 값 | 없음. 모든 필드를 채워야 값이 생김. 기본값은 `Default` | Q6.3 |
| `extends`로 필드와 메서드 물려받기 | 조합과 위임, 트레이트의 기본 메서드 | Q6.4 |
| `super.method()` | `self.base.method()` | Q6.4 |
| Go의 임베딩 | 없음. 손으로 위임 | Q6.4 |
| 인터페이스 | 트레이트(`impl 트레이트 for 타입`으로 명시) | Q6.5 |
| 확장 함수 | 내 트레이트를 남의 타입에 구현 | Q6.5 |
| 제네릭 | 단형성화되는 제네릭과 트레이트 바운드 | Q6.6 |
| 오버로딩 | 다른 이름, 트레이트를 받는 제네릭 인자 | Q6.7 |
| 기본 인자 | `Default`와 구조체 갱신 문법, 빌더 | Q6.7 |
| 가변 인자 | 슬라이스 `&[T]`, 매크로 | Q6.7 |
| `sealed` 계층과 `switch`·`when` | 열거형과 `match` | Q6.8 |
| 열린 계층과 가상 메서드 | 트레이트 객체 `dyn 트레이트` | Q6.8 |
| `instanceof`, 다운캐스트 | `match`, 마지막 수단으로 `Any::downcast_ref` | Q6.8 |
| record, data class, dataclass | `#[derive(Debug, Clone, PartialEq, …)]` | Q6.9 |
| 리플렉션, 어노테이션 처리 | 절차적 매크로(컴파일 시간) | Q6.9 |
| `private`, `protected`, 패키지 공개 | 모듈 단위 비공개, 없음, `pub(crate)` | Q6.10 |

## 3. 예제 안내

| 파일 | 보여 주는 것 | 질문 |
| --- | --- | --- |
| [01_inheritance_to_composition](examples/01_inheritance_to_composition.rs) | Java의 `Employee`/`Manager` 계층을 조합·위임·기본 메서드로 옮기기, 모듈 단위 비공개 | Q6.2, Q6.4, Q6.10 |
| [02_generics_not_erased](examples/02_generics_not_erased.rs) | Java의 타입 소거 때문에 막힌 일에 대응하는 것: `T::default()`, 타입별 크기, 타입 인자만 다른 타입에 다른 `impl` | Q6.6 |
| [03_overloading_alternatives](examples/03_overloading_alternatives.rs) | 생성자 오버로딩·가변 인자·타입별 오버로딩 대신 이름이 다른 함수·슬라이스·트레이트 | Q6.3, Q6.7 |

### 01_inheritance_to_composition — 상속 계층을 조합으로

```sh
cargo run -p faq-ch6-no-classes-inheritance --example 01_inheritance_to_composition
```

```text
1. 기본 메서드와 위임
  김하나: 월 300만 원
  이두리: 월 450만 원
2. 같은 모듈이면 다른 타입의 비공개 필드도 읽음
  이두리의 연간 총액: 5400만 원
```

- 파일 맨 위 주석에 옮기기 전의 Java 코드를 적어 두었습니다.
- `Manager`는 `pay_slip`을 구현하지 않았으므로 `Payable`의 기본 메서드를 씁니다. 기본 메서드 안의 `self.monthly_pay()`는 `Manager`의 구현을 부르므로, Java에서 부모의 `paySlip()`이 재정의된 `monthlyPay()`를 부르는 것과 같은 결과가 됩니다.

### 02_generics_not_erased — 지워지지 않는 타입 인자

```sh
cargo run -p faq-ch6-no-classes-inheritance --example 02_generics_not_erased
```

```text
1. 타입 인자로 새 값 만들기
  u8: 0 (1바이트)
  u64: 0 (8바이트)
  String: "" (24바이트)
2. 타입 인자만 다른 타입에 다른 구현
  바이트 4개
  문자열 2개: 가, 나
```

- `fresh::<u8>()`처럼 함수 이름 뒤에 `::<타입>`을 붙여 타입 인자를 직접 고릅니다. 인자로 추론할 값이 없는 함수라서 필요합니다.
- `String`의 24바이트는 64비트 환경에서 포인터·길이·용량을 담는 크기이고, 문자열 내용은 힙에 있습니다.
- 컴파일러는 타입 인자까지 포함한 전체 타입(`Vec<u8>`, `Vec<String>`)으로 `impl`을 고르므로, 두 타입에 서로 다른 구현을 둘 수 있습니다. `report`는 제네릭 함수라서 `report::<Vec<u8>>`와 `report::<Vec<String>>`이 따로 만들어지고, 각 사본이 해당 `impl`의 `describe`를 바로 부릅니다.

### 03_overloading_alternatives — 오버로딩 대신

```sh
cargo run -p faq-ch6-no-classes-inheritance --example 03_overloading_alternatives
```

```text
1. 생성자 오버로딩 대신 이름이 다른 연관 함수
  차: 180s
  회의: 1800s
2. 가변 인자 대신 슬라이스
  합계: 1980s
  빈 목록: 0ns
3. 인자 타입에 따른 오버로딩 대신 트레이트
  90초 기다림
  2초 기다림
```

- `Duration`의 디버그 형식은 단위를 붙여 보여 줍니다(`180s`, `0ns`).
- `total(&[tea, meeting])`은 두 `Timer`를 배열로 옮겨 그 슬라이스를 넘깁니다. 이후 `tea`와 `meeting`은 쓸 수 없습니다.
- `Duration::from_millis(2500)`의 `as_secs()`는 1초 미만을 버리므로 2초가 됩니다.

## 4. 자주 하는 실수

### 4.1 품은 타입을 부모 타입처럼 넘김 — E0308

파일: [`compile_fail/e0308_composition_is_not_subtype.rs`](compile_fail/e0308_composition_is_not_subtype.rs)

```text
error[E0308]: mismatched types
  --> compile_fail/e0308_composition_is_not_subtype.rs:35:16
   |
35 |     print_name(&lead); // E0308
   |     ---------- ^^^^^ expected `&Employee`, found `&Manager`
   |     |
   |     arguments to this function are incorrect
   |
   = note: expected reference `&Employee`
              found reference `&Manager`
note: function defined here
  --> compile_fail/e0308_composition_is_not_subtype.rs:24:4
   |
24 | fn print_name(employee: &Employee) {
   |    ^^^^^^^^^^ -------------------
```

- **원인**: Java에서 `Manager extends Employee`이면 `Employee`를 받는 메서드에 `Manager`를 넘길 수 있습니다(업캐스트, upcast). 상속을 조합으로 옮긴 뒤에도 같은 습관으로 `&Manager`를 `&Employee` 자리에 넘겼습니다. `Manager`가 `Employee`를 필드로 품고 있어도 둘은 서로 다른 타입입니다. Rust 레퍼런스에 따르면 Rust의 하위 타입 관계(subtyping)는 라이프타임에 관한 경우로 한정되므로, 구조체를 품는 것으로는 생기지 않습니다.
- **해결**: 품은 값을 직접 넘기거나(`print_name(&lead.base)`), 여러 타입을 받아야 하는 함수라면 트레이트로 받습니다(`fn print_name(p: &impl Payable)`, 01 예제의 방식).
- 덧붙임: 트레이트 객체를 슈퍼트레이트의 트레이트 객체로 바꾸는 업캐스트(`&dyn AuditItem` → `&dyn Loanable`)는 1.86.0부터 됩니다([부록 A ch12](../../appd-a/ch12-generics-vs-trait-objects/README.md) 2.4절). 그것은 트레이트 사이의 관계이고, 구조체를 품는 조합과는 다른 이야기입니다.

### 4.2 매개변수만 다른 같은 이름의 연관 함수 — E0592

파일: [`compile_fail/e0592_overloaded_new.rs`](compile_fail/e0592_overloaded_new.rs)

```text
error[E0592]: duplicate definitions with name `new`
  --> compile_fail/e0592_overloaded_new.rs:28:5
   |
20 |     fn new(secs: u64) -> Timer {
   |     -------------------------- other definition for `new`
...
28 |     fn new(label: &str, secs: u64) -> Timer {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ duplicate definitions for `new`
```

- **원인**: Java·C#·C++의 생성자 오버로딩처럼 매개변수만 다른 `new`를 두 개 만들었습니다. Rust는 매개변수 목록이 달라도 한 타입의 고유 `impl`에(블록을 나눠도) 같은 이름의 연관 함수를 둘 둘 수 없습니다. 모듈 수준의 함수라면 같은 실수가 E0428(`the name … is defined multiple times`)이 됩니다.
- **해결**: 만드는 방법을 이름에 담습니다(`new`와 `with_label`, 03 예제의 `new`와 `from_minutes`). `&str`과 `String`처럼 받는 타입만 다르다면 `impl Into<String>` 하나로 받습니다.

## 5. 연습 문제

### 연습 1. 계약직 더하기

01의 `payroll` 모듈에 계약직 `Contractor`를 더해 보세요. 연봉 대신 월 고정액(`monthly`)을 받고, 급여 명세는 기본 메서드 대신 `"박세나: 월 280만 원(계약)"`처럼 끝에 `(계약)`을 붙여 출력합니다. `main`의 목록에도 넣어 세 사람의 명세가 나오게 합니다.

<details>
<summary>힌트</summary>

`Contractor { name: String, monthly: u32 }`와 `pub fn new`를 만들고, `impl Payable for Contractor`에서 `name`, `monthly_pay`와 함께 `pay_slip`도 구현해 기본 메서드를 재정의합니다(Part 0 ch10 2.5절). `use` 줄에 `Contractor`를 더하고, 목록의 타입을 `[&dyn Payable; 3]`으로 바꿉니다. `Contractor`는 `Employee`를 품지 않으므로 위임할 것이 없습니다. Java라면 `super.paySlip() + "(계약)"`으로 쓸 자리지만, 재정의한 `pay_slip` 안에서 기본 구현을 부를 방법은 없으므로(Q6.4) 문장 전체를 다시 씁니다.

</details>

### 연습 2. `Vec<bool>`에도 다른 구현을

02에 `impl Describe for Vec<bool>`을 더해, `report(&vec![true, false, true])`가 `참 2개 / 전체 3개`를 출력하게 해 보세요. 그리고 `fresh::<Vec<u8>>()`와 `fresh::<Option<u8>>()`가 무엇을 출력할지 예상한 뒤 실행해 확인합니다.

<details>
<summary>힌트</summary>

참인 원소 수는 `self.iter().filter(|&&b| b).count()`로 셉니다. Java라면 `describe(List<Boolean>)`를 기존 오버로드와 나란히 둘 수 없는 자리입니다(Q6.6). `fresh::<Vec<u8>>()`는 빈 벡터 `[]`와 24바이트, `fresh::<Option<u8>>()`는 `None`과 2바이트입니다. `Option<u8>`은 `u8`이 가질 수 없는 값이 없어서 `Some`인지 `None`인지를 담을 1바이트가 더 필요합니다.

</details>

### 연습 3. 분과 초로도 기다리기

03의 `IntoSecs`를 튜플 `(u64, u64)`(분, 초)에도 구현해, `wait((1, 30))`이 `90초 기다림`을 출력하게 해 보세요.

<details>
<summary>힌트</summary>

`impl IntoSecs for (u64, u64)` 안에서 `let (minutes, secs) = self;`로 나눈 뒤 `minutes * 60 + secs`를 돌려줍니다. `wait`는 고치지 않아도 됩니다. 오버로딩이라면 `wait(u64, u64)`를 새로 만들었을 자리에, 받을 수 있는 타입 하나를 트레이트 구현으로 더한 것입니다.

</details>

## 6. 참고 자료

확인일은 모두 2026-11-21입니다.

Rust

- The Rust Programming Language: [5.3절 Method Syntax](https://doc.rust-lang.org/book/ch05-03-method-syntax.html), [10.1절 Performance of Code Using Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html#performance-of-code-using-generics), [10.2절 Traits](https://doc.rust-lang.org/book/ch10-02-traits.html), [18.1절 Characteristics of Object-Oriented Languages](https://doc.rust-lang.org/book/ch18-01-what-is-oo.html)
- The Rust Reference: [Visibility and privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html), [Subtyping and Variance](https://doc.rust-lang.org/reference/subtyping.html)
- Rust API Guidelines: [C-CTOR](https://rust-lang.github.io/api-guidelines/predictability.html#c-ctor), [C-DEREF](https://rust-lang.github.io/api-guidelines/predictability.html#c-deref)

다른 언어

- Go: [FAQ — Is Go an object-oriented language?](https://go.dev/doc/faq#Is_Go_an_object-oriented_language), [Why doesn't Go have "implements" declarations?](https://go.dev/doc/faq#implements_interface), [Why does Go not support overloading?](https://go.dev/doc/faq#overloading), [How do generics in Go compare to generics in other languages?](https://go.dev/doc/faq#generics_comparison), [How are generics implemented in Go?](https://go.dev/doc/faq#generics_implementation), 명세의 [Struct types](https://go.dev/ref/spec#Struct_types)(승격), [Method declarations](https://go.dev/ref/spec#Method_declarations), [Exported identifiers](https://go.dev/ref/spec#Exported_identifiers), [Passing arguments to ... parameters](https://go.dev/ref/spec#Passing_arguments_to_..._parameters), [Effective Go — Constructors and composite literals](https://go.dev/doc/effective_go#composite_literals)
- Java: Java Language Specification(SE 21) [4.12.5](https://docs.oracle.com/javase/specs/jls/se21/html/jls-4.html#jls-4.12.5), [12.5](https://docs.oracle.com/javase/specs/jls/se21/html/jls-12.html#jls-12.5), [6.6](https://docs.oracle.com/javase/specs/jls/se21/html/jls-6.html#jls-6.6), [8.4.9](https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html#jls-8.4.9), [8.10.3](https://docs.oracle.com/javase/specs/jls/se21/html/jls-8.html#jls-8.10.3), [9.4](https://docs.oracle.com/javase/specs/jls/se21/html/jls-9.html#jls-9.4), [9.7](https://docs.oracle.com/javase/specs/jls/se21/html/jls-9.html#jls-9.7), [14.11.1.1](https://docs.oracle.com/javase/specs/jls/se21/html/jls-14.html#jls-14.11.1.1). The Java Tutorials: [Restrictions on Generics](https://docs.oracle.com/javase/tutorial/java/generics/restrictions.html), [Bounded Type Parameters](https://docs.oracle.com/javase/tutorial/java/generics/bounded.html)
- C#: [interface](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/interface)
- Kotlin: [Functions — Parameters with default values](https://kotlinlang.org/docs/functions.html), [Extensions](https://kotlinlang.org/docs/extensions.html), [Sealed classes and interfaces](https://kotlinlang.org/docs/sealed-classes.html), [Data classes](https://kotlinlang.org/docs/data-classes.html)
- Python: [`dataclasses`](https://docs.python.org/3/library/dataclasses.html), [용어집 — decorator](https://docs.python.org/3/glossary.html#term-decorator)
