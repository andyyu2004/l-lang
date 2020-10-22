use super::*;

#[test]
fn llvm_tuple_field_access() {
    let out = llvm_exec!("fn main() -> int { (1,8).1 }");
    assert_eq!(out, 8);
}

#[test]
fn llvm_singleton_struct_construction() {
    let src = r#"
    struct S { x: int }

    fn main() -> int {
        S { x: 5 }.x
    }
    "#;
    assert_eq!(llvm_exec!(src), 5)
}

#[test]
fn llvm_tuple_in_struct() {
    let src = r#"
    struct S {
        x: int,
        y: (int, bool, int),
    }

    fn main() -> int {
        S { x: 5, y: (4, false, 9) }.y.2
    }
    "#;
    assert_eq!(llvm_exec!(src), 9)
}

#[test]
fn llvm_nested_struct() {
    let src = r#"
    struct S {
        x: int,
        y: (int, bool, int),
    }

    struct T {
        s: S
    }

    fn main() -> int {
        let s = S {
            x: 5,
            y: (1, false, 3)
        };
        let t = T { s };
        t.s.y.2
    }
    "#;
    assert_eq!(llvm_exec!(src), 3)
}

#[test]
fn llvm_multi_nested_tuples() {
    let src = "fn main() -> int { (1, (2, (3, (4, 5)))).1.1.1.1 }";
    assert_eq!(llvm_exec!(src), 5)
}

#[test]
fn llvm_struct_field_assign() {
    let src = r#"
    struct S { x: int }
    fn main() -> int {
        let s: S = S { x: 4 };
        // intentionally separate these expressions two to ensure s.x has really been assigned to
        s.x = 9;
        s.x
    }"#;
    assert_eq!(llvm_exec!(src), 9)
}

#[test]
fn llvm_recursive_struct() {
    let src = r#"
    struct Node {
        val: int,
            next: NodeOption,
    }

    enum NodeOption {
        None,
        Some(&Node),
    }

    fn main() -> int {
        let node = Node {
            val: 9,
            next: NodeOption::None,
        };

        let head = Node {
            val: 4,
            next: NodeOption::Some(box node),
        };

        match head.next {
            NodeOption::Some(n) => n.val,
            NodeOption::None => 0,
        }
    }"#;

    assert_eq!(llvm_exec!(src), 9)
}
