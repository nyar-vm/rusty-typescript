use typescript::TypeScript;

#[test]
fn test_typescript_features() {
    let mut ts = TypeScript::new();

    // 测试变量声明和类型注解
    let result = ts.execute_script("let x: number = 10; let y: string = 'Hello'; x + y").unwrap();
    println!("Variable declaration result: {:?}", result);

    // 测试函数声明
    let result = ts.execute_script("function add(a: number, b: number): number { return a + b; } add(5, 3)").unwrap();
    println!("Function declaration result: {:?}", result);

    // 测试类声明
    let result = ts.execute_script("class Person { name: string; constructor(name: string) { this.name = name; } greet() { return 'Hello, ' + this.name; } } let p = new Person('Alice'); p.greet()").unwrap();
    println!("Class declaration result: {:?}", result);

    // 测试接口声明
    let result = ts.execute_script("interface Animal { name: string; sound(): string; } let dog: Animal = { name: 'Dog', sound: () => 'Woof!' }; dog.sound()").unwrap();
    println!("Interface declaration result: {:?}", result);

    // 测试类型别名
    let result =
        ts.execute_script("type Point = { x: number; y: number }; let p: Point = { x: 10, y: 20 }; p.x + p.y").unwrap();
    println!("Type alias result: {:?}", result);

    // 测试 for 循环
    let result = ts.execute_script("let sum = 0; for (let i: number = 0; i < 5; i++) { sum += i; } sum").unwrap();
    println!("For loop result: {:?}", result);

    // 测试 while 循环
    let result = ts.execute_script("let count = 0; let sum = 0; while (count < 5) { sum += count; count++; } sum").unwrap();
    println!("While loop result: {:?}", result);

    // 测试 if-else 语句
    let result = ts
        .execute_script("let x: number = 10; if (x > 5) { 'x is greater than 5' } else { 'x is less than or equal to 5' }")
        .unwrap();
    println!("If-else result: {:?}", result);

    // 测试 break 和 continue
    let result = ts.execute_script("let sum = 0; for (let i: number = 0; i < 10; i++) { if (i === 3) { continue; } if (i === 7) { break; } sum += i; } sum").unwrap();
    println!("Break and continue result: {:?}", result);

    // 测试数组
    let result = ts.execute_script("let numbers: number[] = [1, 2, 3, 4, 5]; numbers[2]").unwrap();
    println!("Array result: {:?}", result);

    // 测试对象
    let result = ts.execute_script("let person = { name: 'Bob', age: 30 }; person.name").unwrap();
    println!("Object result: {:?}", result);
}
