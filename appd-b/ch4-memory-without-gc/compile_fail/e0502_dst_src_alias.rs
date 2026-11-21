// 기대하는 에러: E0502
// 핵심 메시지: cannot borrow `v` as immutable because it is also borrowed as mutable
//
// 같은 벡터를 고칠 대상(`&mut`)과 읽을 대상(`&`)으로 동시에 넘깁니다. `append_copy` 안에서 `push`가
// 공간을 늘리며 원소를 새 버퍼로 옮기면 옛 버퍼가 해제될 수 있는데, `src`는 바로 그 옛 버퍼를 읽고
// 있으므로 해제 후 사용이 됩니다. 옮길지는 실행 중에야 정해지므로, 가변 참조가 살아 있는 동안 같은
// 값의 다른 참조를 만들 수 없다는 규칙으로 언제나 막습니다.
//
// 고치는 법: `v.extend_from_within(..)`처럼 이 일을 안전하게 해 주는 메서드를 쓰거나, 사본을 먼저
// 만들어 넘깁니다(`let snapshot = v.clone(); append_copy(&mut v, &snapshot);`). 예제 02 참고.
//
// 확인(챕터 디렉터리에서):
//   rustc --edition 2024 --out-dir ../../target/tmp/faq-ch4-memory-without-gc compile_fail/e0502_dst_src_alias.rs

fn append_copy(dst: &mut Vec<i32>, src: &[i32]) {
    for &x in src {
        dst.push(x);
    }
}

fn main() {
    let mut v = vec![1, 2, 3];
    append_copy(&mut v, &v); // E0502
    println!("{v:?}");
}
