// 단언 매크로 assert!, assert_eq!, assert_ne!와 사용자 정의 실패 메시지를 보여 줍니다.
// assert_eq!로 직접 만든 타입을 비교하려면 PartialEq와 Debug가 필요하다는 것도 확인합니다.
//
// main 실행:   cargo run -p p0-ch11-testing --example 01_assertions
// 테스트 실행: cargo test -p p0-ch11-testing --example 01_assertions
// 실패 보기:   cargo test -p p0-ch11-testing --example 01_assertions -- --ignored
//
// 맨 아래 failure_demos 모듈의 테스트는 일부러 틀린 기대값을 적은 것이라 반드시 실패합니다.
// 평소에는 #[ignore]로 건너뛰고, 실패 메시지를 보고 싶을 때만 -- --ignored로 실행합니다.

// assert_eq!(a, b)는 a == b로 비교하고(PartialEq), 실패하면 두 값을 {:?}로 출력합니다(Debug).
// 그래서 assert_eq!로 비교할 타입에는 두 트레이트가 모두 있어야 합니다.
#[derive(Debug, PartialEq)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn square(size: u32) -> Rectangle {
        Rectangle {
            width: size,
            height: size,
        }
    }

    // 다른 사각형이 이 사각형 안에 완전히 들어가는지 검사합니다.
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

fn add_two(a: i32) -> i32 {
    a + 2
}

fn greeting(name: &str) -> String {
    format!("안녕하세요, {name}님!")
}

// main은 테스트할 함수들이 실제로 무엇을 돌려주는지 보여 줍니다.
// cargo test로 실행하면 main은 실행되지 않고, 아래 #[test] 함수들만 실행됩니다.
fn main() {
    let larger = Rectangle {
        width: 8,
        height: 7,
    };
    let smaller = Rectangle {
        width: 5,
        height: 1,
    };
    println!("larger.can_hold(&smaller) = {}", larger.can_hold(&smaller)); // 출력: larger.can_hold(&smaller) = true
    println!("smaller.can_hold(&larger) = {}", smaller.can_hold(&larger)); // 출력: smaller.can_hold(&larger) = false
    println!("add_two(2) = {}", add_two(2)); // 출력: add_two(2) = 4
    println!("{:?}", Rectangle::square(3)); // 출력: Rectangle { width: 3, height: 3 }
    println!("{}", greeting("영희")); // 출력: 안녕하세요, 영희님!
}

// #[cfg(test)]: cargo test로 빌드할 때만 이 모듈을 컴파일합니다.
// cargo run이나 cargo build로 만든 실행 파일에는 테스트 코드가 들어가지 않습니다.
#[cfg(test)]
mod tests {
    // tests는 자식 모듈이므로, 부모(이 파일의 크레이트 루트)의 항목을 쓰려면 가져와야 합니다.
    // 자식 모듈은 조상의 비공개 항목도 볼 수 있으므로 pub이 없는 함수도 테스트할 수 있습니다.
    use super::*;

    // assert!(조건): 조건이 true면 통과, false면 패닉합니다. bool을 돌려주는 함수에 알맞습니다.
    #[test]
    fn larger_can_hold_smaller() {
        // 준비: 테스트에 쓸 값을 만듭니다.
        let larger = Rectangle {
            width: 8,
            height: 7,
        };
        let smaller = Rectangle {
            width: 5,
            height: 1,
        };

        // 실행과 단언: 결과가 기대와 같은지 확인합니다.
        assert!(larger.can_hold(&smaller));
    }

    #[test]
    fn smaller_cannot_hold_larger() {
        let larger = Rectangle {
            width: 8,
            height: 7,
        };
        let smaller = Rectangle {
            width: 5,
            height: 1,
        };

        // false여야 하는 결과는 !로 뒤집어서 확인합니다.
        assert!(!smaller.can_hold(&larger));
    }

    // assert_eq!(왼쪽, 오른쪽): 두 값이 같아야 통과합니다.
    // 실패하면 두 값을 모두 보여 주므로 assert!(a == b)보다 원인을 찾기 쉽습니다.
    #[test]
    fn add_two_adds_two() {
        assert_eq!(add_two(2), 4);
    }

    // assert_ne!(왼쪽, 오른쪽): 두 값이 달라야 통과합니다.
    // 정확한 값은 몰라도 "적어도 이 값은 아니어야 한다"는 것을 확인할 때 씁니다.
    #[test]
    fn add_two_changes_value() {
        assert_ne!(add_two(5), 5);
    }

    // 직접 만든 타입도 PartialEq와 Debug가 있으면 assert_eq!로 비교할 수 있습니다.
    #[test]
    fn square_has_same_sides() {
        let expected = Rectangle {
            width: 3,
            height: 3,
        };
        assert_eq!(Rectangle::square(3), expected);
    }

    // 단언 매크로의 필수 인자 뒤에 쓴 인자는 format!처럼 실패 메시지가 됩니다.
    // 통과할 때는 메시지를 만들지 않으므로, 실패 원인을 알려 주는 값을 넉넉히 넣어도 됩니다.
    #[test]
    fn greeting_contains_name() {
        let result = greeting("영희");
        assert!(
            result.contains("영희"),
            "인사말에 이름이 없습니다. 실제 값: `{result}`"
        );
    }

    // 아래 테스트들은 실패 메시지를 보여 주기 위해 기대값을 일부러 틀리게 적었습니다.
    // #[ignore = "이유"]를 붙인 테스트는 평소 cargo test에서 건너뛰고, 출력에 이유를 함께 보여 줍니다.
    mod failure_demos {
        use super::*;

        // assert!는 조건식 자체를 메시지로 보여 줍니다. 값은 보여 주지 않습니다.
        #[test]
        #[ignore = "실패 데모"]
        fn assert_fails() {
            let larger = Rectangle {
                width: 8,
                height: 7,
            };
            let smaller = Rectangle {
                width: 5,
                height: 1,
            };
            assert!(smaller.can_hold(&larger));
        }

        // assert_eq!는 두 값을 Debug 형식으로 보여 줍니다.
        #[test]
        #[ignore = "실패 데모"]
        fn assert_eq_fails() {
            assert_eq!(Rectangle::square(3), Rectangle::square(4));
        }

        // assert!에 준 사용자 정의 메시지는 기본 메시지("assertion failed: ...") 대신 출력됩니다.
        #[test]
        #[ignore = "실패 데모"]
        fn custom_message_fails() {
            let result = greeting("영희");
            assert!(
                result.contains("철수"),
                "인사말에 이름이 없습니다. 실제 값: `{result}`"
            );
        }

        // assert_eq!에 준 메시지는 기본 메시지 뒤에 붙고, 두 값도 그대로 출력됩니다.
        #[test]
        #[ignore = "실패 데모"]
        fn assert_eq_with_message_fails() {
            let input = 2;
            assert_eq!(add_two(input), 5, "add_two({input})의 결과가 틀렸습니다");
        }
    }
}
