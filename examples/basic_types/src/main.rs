use typescript::TypeScript;

fn main() {
    let mut ts = TypeScript::new();
    
    // 测试基本类型
    match ts.execute_script("let x = 10; let y = 'hello'; console.log(x, y); x + y") {
        Ok(result) => {
            println!("Test result: {:?}", result);
        },
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
}
