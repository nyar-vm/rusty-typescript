use typescript_ir::{Expression, Program, Statement, optimize_basic};
use typescript_types::TsValue;

#[test]
fn test_constant_folding() {
    let expr = Expression::Binary {
        left: Box::new(Expression::Literal(TsValue::Number(1.0))),
        op: typescript_ir::BinaryOp::Add,
        right: Box::new(Expression::Literal(TsValue::Number(2.0))),
    };

    let program = Program { statements: vec![Statement::Expression(Box::new(expr))] };

    let optimized = optimize_basic(&program);

    if let Statement::Expression(expr) = &optimized.statements[0] {
        if let Expression::Literal(value) = expr.as_ref() {
            assert!(value.is_number());
            assert_eq!(value.to_number(), 3.0);
        }
        else {
            panic!("Expected Literal, got {:?}", expr);
        }
    }
    else {
        panic!("Expected Expression statement, got {:?}", optimized.statements[0]);
    }
}

#[test]
fn test_dead_code_elimination() {
    let program = Program {
        statements: vec![
            Statement::Return(Some(Box::new(Expression::Literal(TsValue::Number(42.0))))),
            Statement::Expression(Box::new(Expression::Literal(TsValue::Number(100.0)))),
        ],
    };

    let optimized = optimize_basic(&program);

    assert_eq!(optimized.statements.len(), 1);
    assert!(matches!(&optimized.statements[0], Statement::Return(_)));
}

#[test]
fn test_while_optimization() {
    let program = Program {
        statements: vec![Statement::While {
            test: Box::new(Expression::Literal(TsValue::Boolean(false))),
            body: Box::new(Statement::Expression(Box::new(Expression::Literal(TsValue::Number(1.0))))),
        }],
    };

    let optimized = optimize_basic(&program);

    assert_eq!(optimized.statements.len(), 0);
}

#[test]
fn test_conditional_optimization() {
    let expr = Expression::Conditional {
        test: Box::new(Expression::Literal(TsValue::Boolean(true))),
        consequent: Box::new(Expression::Literal(TsValue::Number(1.0))),
        alternate: Box::new(Expression::Literal(TsValue::Number(2.0))),
    };

    let program = Program { statements: vec![Statement::Expression(Box::new(expr))] };

    let optimized = optimize_basic(&program);

    if let Statement::Expression(expr) = &optimized.statements[0] {
        if let Expression::Literal(value) = expr.as_ref() {
            assert!(value.is_number());
            assert_eq!(value.to_number(), 1.0);
        }
        else {
            panic!("Expected Literal, got {:?}", expr);
        }
    }
    else {
        panic!("Expected Expression statement, got {:?}", optimized.statements[0]);
    }
}

#[test]
fn test_unary_optimization() {
    let expr =
        Expression::Unary { op: typescript_ir::UnaryOp::Not, expr: Box::new(Expression::Literal(TsValue::Boolean(true))) };

    let program = Program { statements: vec![Statement::Expression(Box::new(expr))] };

    let optimized = optimize_basic(&program);

    if let Statement::Expression(expr) = &optimized.statements[0] {
        if let Expression::Literal(value) = expr.as_ref() {
            assert!(value.is_boolean());
            assert_eq!(value.to_boolean(), false);
        }
        else {
            panic!("Expected Literal, got {:?}", expr);
        }
    }
    else {
        panic!("Expected Expression statement, got {:?}", optimized.statements[0]);
    }
}
