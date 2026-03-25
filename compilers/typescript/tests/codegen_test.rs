use crate::compilers::typescript::TypeScript;

#[test]
fn test_arithmetic_operations() {
    let mut ts = TypeScript::new();
    let arithmetic_test = r#"
        let a = 10;
        let b = 5;
        let sum = a + b;
        let difference = a - b;
        let product = a * b;
        let quotient = a / b;
        let remainder = a % b;
        console.log('Sum:', sum);
        console.log('Difference:', difference);
        console.log('Product:', product);
        console.log('Quotient:', quotient);
        console.log('Remainder:', remainder);
    "#;

    let result = ts.execute_script(arithmetic_test);
    assert!(result.is_ok(), "Arithmetic test failed: {:?}", result);
}

#[test]
fn test_logical_operations() {
    let mut ts = TypeScript::new();
    let logical_test = r#"
        let x = true;
        let y = false;
        let and_result = x && y;
        let or_result = x || y;
        let not_result = !x;
        console.log('And result:', and_result);
        console.log('Or result:', or_result);
        console.log('Not result:', not_result);
    "#;

    let result = ts.execute_script(logical_test);
    assert!(result.is_ok(), "Logical test failed: {:?}", result);
}

#[test]
fn test_control_flow() {
    let mut ts = TypeScript::new();
    let control_flow_test = r#"
        let i = 0;
        while (i < 5) {
            console.log('Loop iteration:', i);
            i = i + 1;
        }
        
        if (i > 3) {
            console.log('i is greater than 3');
        } else {
            console.log('i is not greater than 3');
        }
    "#;

    let result = ts.execute_script(control_flow_test);
    assert!(result.is_ok(), "Control flow test failed: {:?}", result);
}
