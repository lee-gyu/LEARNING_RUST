// 순환 참조가 메모리 누수를 만드는 모습을 관찰하고, Weak<T>로 순환 없이 서로를 가리키는 트리를 만듭니다.
// - Rc끼리 서로를 가리키면 strong count가 0이 되지 않아 값이 영영 drop되지 않습니다.
// - Weak<T>는 카운트(weak count)만 남기고 값을 살려 두지 않습니다. upgrade()로 아직 있는지 확인합니다.
// - 트리에서 부모 → 자식은 Rc(소유), 자식 → 부모는 Weak(소유하지 않음)로 둡니다.
//
// 실행: cargo run -p p0-ch14-smart-pointers --example 08_cycle_and_weak

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// ---- 1~2번 구역: 꼬리를 바꿀 수 있는 cons 리스트 ----
// 꼬리를 RefCell<Rc<List>>로 두었기 때문에, 이미 만든 리스트의 다음 칸을 나중에 바꿀 수 있습니다.
#[derive(Debug)]
enum List {
    Cons(i32, RefCell<Rc<List>>),
    Nil,
}

use crate::List::{Cons, Nil};

impl List {
    fn tail(&self) -> Option<&RefCell<Rc<List>>> {
        match self {
            Cons(_, item) => Some(item),
            Nil => None,
        }
    }
}

// 버려지는지 보려고 Drop을 구현합니다. Nil은 아무것도 출력하지 않습니다.
impl Drop for List {
    fn drop(&mut self) {
        if let Cons(value, _) = self {
            println!("    [drop] Cons({value})");
        }
    }
}

// ---- 3~6번 구역: 부모를 Weak로 가리키는 트리 노드 ----
#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,      // 자식 → 부모: 소유하지 않습니다.
    children: RefCell<Vec<Rc<Node>>>, // 부모 → 자식: 소유합니다.
}

impl Drop for Node {
    fn drop(&mut self) {
        println!("    [drop] Node({})", self.value);
    }
}

// 부모도 자식도 없는 노드를 만듭니다. Weak::new()는 아무것도 가리키지 않는 Weak입니다.
fn node(value: i32) -> Rc<Node> {
    Rc::new(Node {
        value,
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    })
}

// 부모의 값을 Option으로 돌려줍니다. 부모가 없거나 이미 버려졌으면 None입니다.
fn parent_value(node: &Node) -> Option<i32> {
    node.parent.borrow().upgrade().map(|parent| parent.value)
}

fn print_counts(name: &str, node: &Rc<Node>) {
    println!(
        "    {name}: strong = {}, weak = {}",
        Rc::strong_count(node),
        Rc::weak_count(node)
    );
}

fn main() {
    println!("1. 비교용: 순환이 없으면 스코프가 끝날 때 정상적으로 drop됩니다");
    {
        let a = Rc::new(Cons(5, RefCell::new(Rc::new(Nil))));
        let b = Rc::new(Cons(10, RefCell::new(Rc::clone(&a)))); // b → a
        println!("    a의 카운트 = {}", Rc::strong_count(&a)); // 출력: a의 카운트 = 2
        println!("    b의 카운트 = {}", Rc::strong_count(&b)); // 출력: b의 카운트 = 1
    } // 변수 b가 먼저 버려집니다. b의 카운트가 0이 되어 Cons(10)이 drop되고,
    // 그 안의 Rc(a를 가리킴)가 버려져 a의 카운트가 1이 됩니다. 이어서 변수 a가 버려져 0이 됩니다.
    // 출력:
    //     [drop] Cons(10)
    //     [drop] Cons(5)

    println!("2. Rc로 순환을 만들면 스코프가 끝나도 drop되지 않습니다 (메모리 누수)");
    {
        let a = Rc::new(Cons(5, RefCell::new(Rc::new(Nil))));
        let b = Rc::new(Cons(10, RefCell::new(Rc::clone(&a)))); // b → a
        println!("    b의 다음 칸 = {:?}", b.tail()); // 출력: b의 다음 칸 = Some(RefCell { value: Cons(5, RefCell { value: Nil }) })

        // a의 꼬리를 Nil에서 b로 바꿉니다. 이제 a → b → a → b ... 순환입니다.
        if let Some(link) = a.tail() {
            *link.borrow_mut() = Rc::clone(&b);
        } // 이 대입으로 원래 꼬리였던 Nil이 버려집니다(Nil은 출력하지 않습니다).
        println!("    순환을 만든 뒤 a의 카운트 = {}", Rc::strong_count(&a)); // 출력: 순환을 만든 뒤 a의 카운트 = 2
        println!("    순환을 만든 뒤 b의 카운트 = {}", Rc::strong_count(&b)); // 출력: 순환을 만든 뒤 b의 카운트 = 2

        // 주의: 아래 줄의 주석을 풀면 디버그 출력이 순환을 끝없이 따라가다 스택 오버플로로
        // 비정상 종료합니다(`thread 'main' ... has overflowed its stack`).
        // println!("a의 다음 칸 = {:?}", a.tail());
    } // 변수 b, a가 버려지며 각 카운트가 2 → 1이 됩니다. 0이 아니므로 아무것도 drop되지 않습니다.
    println!("    블록이 끝났지만 [drop] Cons(..) 줄이 찍히지 않았습니다");
    // 두 칸은 서로를 붙잡은 채 프로그램이 끝날 때까지 메모리에 남습니다.
    // (프로세스가 끝나면 운영체제가 메모리를 회수하지만, 그 전까지는 아무도 해제하지 못합니다.)

    println!("3. Weak는 값을 가리키기만 하고 살려 두지는 않습니다");
    let owner = node(1);
    let weak: Weak<Node> = Rc::downgrade(&owner); // strong이 아니라 weak 카운트가 늘어납니다.
    print_counts("owner", &owner); // 출력: owner: strong = 1, weak = 1
    // upgrade()는 값이 살아 있으면 Some(Rc<T>)를 돌려줍니다. 돌려받은 Rc도 소유자이므로 strong이 늘어납니다.
    if let Some(upgraded) = weak.upgrade() {
        println!("    upgrade → Some(Node({}))", upgraded.value); // 출력: upgrade → Some(Node(1))
        print_counts("owner", &owner); // 출력: owner: strong = 2, weak = 1
    } // upgraded가 버려지며 strong이 다시 1이 됩니다.
    drop(owner); // 마지막 Rc가 사라집니다. weak가 남아 있어도 값은 drop됩니다.
    println!(
        "    owner를 버린 뒤 upgrade → {:?}",
        weak.upgrade().map(|n| n.value)
    );
    // 출력:
    //     [drop] Node(1)
    //     owner를 버린 뒤 upgrade → None

    println!("4. 트리: 자식은 부모를 Weak로 가리킵니다");
    let leaf = node(3);
    println!("    leaf의 부모 = {:?}", parent_value(&leaf)); // 출력: leaf의 부모 = None
    print_counts("leaf", &leaf); // 출력: leaf: strong = 1, weak = 0
    {
        let branch = Rc::new(Node {
            value: 5,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![Rc::clone(&leaf)]), // branch가 leaf를 소유합니다.
        });
        // leaf의 부모 칸에 branch를 Weak로 넣습니다. 서로를 가리키지만 순환 소유는 아닙니다.
        *leaf.parent.borrow_mut() = Rc::downgrade(&branch);

        println!("    leaf의 부모 = {:?}", parent_value(&leaf)); // 출력: leaf의 부모 = Some(5)
        println!("    branch의 자식 수 = {}", branch.children.borrow().len()); // 출력: branch의 자식 수 = 1
        print_counts("branch", &branch); // 출력: branch: strong = 1, weak = 1
        print_counts("leaf", &leaf); // 출력: leaf: strong = 2, weak = 0
        // 디버그 형식은 Weak를 (Weak)로만 표시하고 따라가지 않으므로 끝없이 출력되지 않습니다.
        println!("    leaf = {leaf:?}"); // 출력: leaf = Node { value: 3, parent: RefCell { value: (Weak) }, children: RefCell { value: [] } }

        println!("5. 부모가 사라지면 자식의 Weak는 None이 됩니다");
    } // branch의 strong이 1 → 0이 되어 drop됩니다. branch가 소유하던 leaf의 Rc도 함께 버려집니다.
    // 출력: [drop] Node(5)
    println!("    leaf의 부모 = {:?}", parent_value(&leaf)); // 출력: leaf의 부모 = None
    print_counts("leaf", &leaf); // 출력: leaf: strong = 1, weak = 0

    println!("6. main이 끝나면 남은 leaf도 drop됩니다");
} // 출력: [drop] Node(3)
