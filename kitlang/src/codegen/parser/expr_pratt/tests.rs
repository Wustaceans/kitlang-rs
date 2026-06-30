// Unit tests for the Pratt parser. Included into `mod.rs` via `include!`,
// so this file is in the same module scope. Only compiled in test builds.

use crate::codegen::types::{AssignmentOperator, BinaryOperator, UnaryOperator};

/// Convenience: parse an expression and unwrap.
fn p(text: &str) -> Expr {
    parse_kit_expr(text).unwrap_or_else(|e| panic!("parse failed for `{text}`: {e}"))
}

/// Convenience: parse and assert the error contains a substring.
fn p_err(text: &str, needle: &str) {
    let err = parse_kit_expr(text)
        .err()
        .unwrap_or_else(|| panic!("expected error for `{text}`, got Ok"));
    let msg = err.to_human_message();
    assert!(
        msg.contains(needle),
        "error `{msg}` does not contain `{needle}`"
    );
}

// --- Literals ---

#[test]
fn integer_literal() {
    let e = p("42");
    assert!(matches!(
        e,
        Expr::Literal {
            value: Literal::Int(42),
            ..
        }
    ));
}

#[test]
fn float_literal() {
    let e = p("3.14");
    assert!(
        matches!(e, Expr::Literal { value: Literal::Float(f), .. } if (f - 3.14).abs() < 1e-10)
    );
}

#[test]
fn string_literal() {
    let e = p(r#""hello""#);
    assert!(matches!(e, Expr::Literal { value: Literal::String(s), .. } if s == "hello"));
}

#[test]
fn bool_literals() {
    assert!(matches!(
        p("true"),
        Expr::Literal {
            value: Literal::Bool(true),
            ..
        }
    ));
    assert!(matches!(
        p("false"),
        Expr::Literal {
            value: Literal::Bool(false),
            ..
        }
    ));
}

#[test]
fn null_literal() {
    assert!(matches!(
        p("null"),
        Expr::Literal {
            value: Literal::Null,
            ..
        }
    ));
}

// --- Identifiers ---

#[test]
fn identifier() {
    let e = p("foo");
    assert!(matches!(&e, Expr::Identifier { name, .. } if name == "foo"));
}

#[test]
fn qualified_identifier_is_built_via_postfix_chain() {
    let e = p("foo.bar.baz");
    let mut cur = &e;
    let mut path = vec![];
    while let Expr::FieldAccess {
        expr, field_name, ..
    } = cur
    {
        path.push(field_name.clone());
        cur = expr;
    }
    if let Expr::Identifier { name, .. } = cur {
        assert_eq!(name, "foo");
    } else {
        panic!("expected leaf Identifier, got {cur:?}");
    }
    assert_eq!(path, vec!["baz".to_string(), "bar".to_string()]);
}

// --- Precedence ---

#[test]
fn additive_vs_multiplicative() {
    let e = p("1 + 2 * 3");
    if let Expr::BinaryOp { op, right, .. } = &e {
        assert_eq!(*op, BinaryOperator::Add);
        if let Expr::BinaryOp { op: inner_op, .. } = right.as_ref() {
            assert_eq!(*inner_op, BinaryOperator::Mul);
        } else {
            panic!("expected inner Mul, got {right:?}");
        }
    } else {
        panic!("expected top-level Add, got {e:?}");
    }
}

#[test]
fn comparison_vs_equality() {
    let e = p("a == b < c");
    if let Expr::BinaryOp {
        op, left, right, ..
    } = &e
    {
        assert_eq!(*op, BinaryOperator::Eq);
        assert!(matches!(left.as_ref(), Expr::Identifier { name, .. } if name == "a"));
        if let Expr::BinaryOp { op: inner_op, .. } = right.as_ref() {
            assert_eq!(*inner_op, BinaryOperator::Lt);
        } else {
            panic!("expected inner Lt, got {right:?}");
        }
    } else {
        panic!("expected top-level Eq, got {e:?}");
    }
}

#[test]
fn left_associative_addition() {
    let e = p("1 + 2 + 3");
    if let Expr::BinaryOp {
        op, left, right, ..
    } = &e
    {
        assert_eq!(*op, BinaryOperator::Add);
        assert!(matches!(
            right.as_ref(),
            Expr::Literal {
                value: Literal::Int(3),
                ..
            }
        ));
        if let Expr::BinaryOp { op: inner_op, .. } = left.as_ref() {
            assert_eq!(*inner_op, BinaryOperator::Add);
        } else {
            panic!("expected inner Add, got {left:?}");
        }
    } else {
        panic!("expected top-level Add, got {e:?}");
    }
}

#[test]
fn right_associative_assignment() {
    let e = p("a += b += c");
    if let Expr::Assign {
        op, left, right, ..
    } = &e
    {
        assert_eq!(*op, AssignmentOperator::AddAssign);
        assert!(matches!(left.as_ref(), Expr::Identifier { name, .. } if name == "a"));
        assert!(matches!(right.as_ref(), Expr::Assign { .. }));
    } else {
        panic!("expected top-level Assign, got {e:?}");
    }
}

#[test]
fn unary_minus_binds_tighter_than_addition() {
    let e = p("-a + b");
    if let Expr::BinaryOp {
        op, left, right, ..
    } = &e
    {
        assert_eq!(*op, BinaryOperator::Add);
        assert!(matches!(right.as_ref(), Expr::Identifier { name, .. } if name == "b"));
        assert!(matches!(
            left.as_ref(),
            Expr::UnaryOp {
                op: UnaryOperator::Neg,
                ..
            }
        ));
    } else {
        panic!("expected top-level Add, got {e:?}");
    }
}

#[test]
fn unary_looser_than_postfix() {
    let e = p("&arr[i]");
    if let Expr::UnaryOp { op, expr, .. } = &e {
        assert_eq!(*op, UnaryOperator::AddressOf);
        assert!(matches!(expr.as_ref(), Expr::Index { .. }));
    } else {
        panic!("expected top-level AddressOf, got {e:?}");
    }
}

// --- Postfix chains ---

#[test]
fn chained_field_access() {
    let e = p("a.b.c.d.e");
    let mut depth = 0;
    let mut cur = &e;
    while let Expr::FieldAccess { expr, .. } = cur {
        depth += 1;
        cur = expr;
    }
    assert_eq!(depth, 4, "expected 4 field-access levels");
    assert!(matches!(cur, Expr::Identifier { name, .. } if name == "a"));
}

#[test]
fn stress_deep_postfix_chain() {
    let mut src = String::from("a");
    for i in 0..100 {
        src.push('.');
        src.push_str(&format!("f{i}"));
    }
    let e = p(&src);
    let mut depth = 0;
    let mut cur = &e;
    while let Expr::FieldAccess { expr, .. } = cur {
        depth += 1;
        cur = expr;
    }
    assert_eq!(depth, 100);
}

#[test]
fn stress_deep_nested_parens() {
    let mut src = String::new();
    for _ in 0..100 {
        src.push('(');
    }
    src.push('1');
    for _ in 0..100 {
        src.push(')');
    }
    let e = p(&src);
    assert!(matches!(
        e,
        Expr::Literal {
            value: Literal::Int(1),
            ..
        }
    ));
}

// --- Function calls ---

#[test]
fn call_no_args() {
    let e = p("f()");
    if let Expr::Call { callee, args, .. } = &e {
        assert_eq!(callee, "f");
        assert!(args.is_empty());
    } else {
        panic!("expected Call, got {e:?}");
    }
}

#[test]
fn call_one_arg() {
    let e = p("f(1)");
    if let Expr::Call { callee, args, .. } = &e {
        assert_eq!(callee, "f");
        assert_eq!(args.len(), 1);
    } else {
        panic!("expected Call, got {e:?}");
    }
}

#[test]
fn call_many_args() {
    let e = p("f(1, 2, 3, 4, 5)");
    if let Expr::Call { args, .. } = &e {
        assert_eq!(args.len(), 5);
    } else {
        panic!("expected Call, got {e:?}");
    }
}

#[test]
fn call_qualified_name() {
    let e = p("pkg.math.add(2, 3)");
    if let Expr::Call { callee, args, .. } = &e {
        assert_eq!(callee, "pkg.math.add");
        assert_eq!(args.len(), 2);
    } else {
        panic!("expected Call, got {e:?}");
    }
}

#[test]
fn call_with_nested_expressions_in_args() {
    let e = p("f(g(1), h(2, 3))");
    if let Expr::Call { args, .. } = &e {
        assert_eq!(args.len(), 2);
    } else {
        panic!("expected Call, got {e:?}");
    }
}

// --- Indexing ---

#[test]
fn index() {
    let e = p("arr[0]");
    if let Expr::Index { expr, index, .. } = &e {
        assert!(matches!(expr.as_ref(), Expr::Identifier { name, .. } if name == "arr"));
        assert!(matches!(
            index.as_ref(),
            Expr::Literal {
                value: Literal::Int(0),
                ..
            }
        ));
    } else {
        panic!("expected Index, got {e:?}");
    }
}

#[test]
fn chained_index() {
    let e = p("a[i][j]");
    let mut depth = 0;
    let mut cur = &e;
    while let Expr::Index { expr, .. } = cur {
        depth += 1;
        cur = expr;
    }
    assert_eq!(depth, 2);
}

// --- Array literals ---

#[test]
fn empty_array() {
    let e = p("[]");
    if let Expr::ArrayLiteral { elements, .. } = &e {
        assert!(elements.is_empty());
    } else {
        panic!("expected ArrayLiteral, got {e:?}");
    }
}

#[test]
fn array_with_elements() {
    let e = p("[1, 2, 3]");
    if let Expr::ArrayLiteral { elements, .. } = &e {
        assert_eq!(elements.len(), 3);
    } else {
        panic!("expected ArrayLiteral, got {e:?}");
    }
}

// --- Struct init ---

#[test]
fn struct_init() {
    let e = p("struct Point { x: 10, y: 20 }");
    if let Expr::StructInit { fields, .. } = &e {
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "x");
        assert_eq!(fields[1].name, "y");
    } else {
        panic!("expected StructInit, got {e:?}");
    }
}

// --- If expressions ---

#[test]
fn if_expr() {
    let e = p("if a then b else c");
    if let Expr::If {
        cond,
        then_branch,
        else_branch,
        ..
    } = &e
    {
        assert!(matches!(cond.as_ref(), Expr::Identifier { name, .. } if name == "a"));
        assert!(matches!(then_branch.as_ref(), Expr::Identifier { name, .. } if name == "b"));
        assert!(matches!(else_branch.as_ref(), Expr::Identifier { name, .. } if name == "c"));
    } else {
        panic!("expected If, got {e:?}");
    }
}

// --- Logical operators ---

#[test]
fn logical_and_vs_or() {
    let e = p("a || b && c");
    if let Expr::BinaryOp { op, right, .. } = &e {
        assert_eq!(*op, BinaryOperator::Or);
        assert!(matches!(
            right.as_ref(),
            Expr::BinaryOp {
                op: BinaryOperator::And,
                ..
            }
        ));
    } else {
        panic!("expected top-level Or, got {e:?}");
    }
}

// --- Errors ---

#[test]
fn missing_rparen() {
    p_err("(1 + 2", "`)`");
}

// --- Range literals ---

#[test]
fn range_literal_simple() {
    let e = p("1...5");
    if let Expr::RangeLiteral { start, end } = &e {
        assert!(matches!(
            start.as_ref(),
            Expr::Literal {
                value: Literal::Int(1),
                ..
            }
        ));
        assert!(matches!(
            end.as_ref(),
            Expr::Literal {
                value: Literal::Int(5),
                ..
            }
        ));
    } else {
        panic!("expected RangeLiteral, got {e:?}");
    }
}

#[test]
fn range_literal_with_expressions() {
    let e = p("a + 1...b - 1");
    if let Expr::RangeLiteral { start, end } = &e {
        assert!(matches!(
            start.as_ref(),
            Expr::BinaryOp {
                op: BinaryOperator::Add,
                ..
            }
        ));
        assert!(matches!(
            end.as_ref(),
            Expr::BinaryOp {
                op: BinaryOperator::Sub,
                ..
            }
        ));
    } else {
        panic!("expected RangeLiteral, got {e:?}");
    }
}

#[test]
fn missing_rbracket() {
    p_err("arr[0", "`]`");
}

#[test]
fn unexpected_token_at_start() {
    p_err("+", "identifier");
}

#[test]
fn missing_field_name() {
    p_err("foo.", "identifier");
}

#[test]
fn missing_else() {
    p_err("if a then b", "`else`");
}
