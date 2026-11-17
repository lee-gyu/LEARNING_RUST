// 기대하는 에러: error[E0507]: cannot move out of `value`, a captured variable in an `FnMut` closure
// 핵심 메시지: captured by this `FnMut` closure / `value` is moved here
//             help: `Fn` and `FnMut` closures require captured values to be able to be consumed multiple times,
//                   but `FnOnce` closures may consume them only once
//
// 확인 (챕터 디렉터리에서):
//   rustc --edition 2024 compile_fail/e0507_move_out_in_fnmut.rs --out-dir /tmp/rust-ch13
//
// 정렬하면서 클로저가 몇 번 불리는지 세려고, 캡처한 value를 벡터에 넣는(이동하는) 경우입니다.
// 캡처한 값을 밖으로 옮기는 클로저는 한 번만 부를 수 있으므로 FnOnce만 구현합니다. 그런데 sort_by_key는
// 원소마다 클로저를 여러 번 부르므로 FnMut을 요구합니다. 두 번째 호출 때는 옮길 value가 없기 때문입니다.
// 고치는 방법: 값을 옮기지 않도록 본문을 바꿉니다. 횟수를 세려면 바깥의 정수 카운터를 += 1 하면 됩니다
// (examples/03_fn_traits.rs의 6번 구역).

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

fn main() {
    let mut list = [
        Rectangle {
            width: 10,
            height: 1,
        },
        Rectangle {
            width: 3,
            height: 5,
        },
        Rectangle {
            width: 7,
            height: 12,
        },
    ];

    let mut sort_operations = vec![];
    let value = String::from("closure called");

    list.sort_by_key(|r| {
        sort_operations.push(value);
        r.width
    });
    println!("{list:#?}");
}
