/// 调试工具
use crate::parse;

/// 调试解析行为
pub fn debug_parse() {
    // 尝试不同的 WebIDL 格式
    let webidl1 = r#"interface TestInterface { void test(); };"#;
    let webidl2 = r#"interface TestInterface {
    void test();
};
"#;
    let webidl3 = r#"module TestModule {
    interface TestInterface {
        void test();
    };
};
"#;

    let webidls = vec![("Simple", webidl1), ("Formatted", webidl2), ("Module", webidl3)];

    for (name, webidl) in webidls {
        println!("\nTesting typescript_webidl parser with {} format...", name);
        println!("Input: {}", webidl);
        
        match parse(webidl) {
            Ok(root) => {
                println!("Parsing successful!");
                println!("Items count: {}", root.items.len());
                for (i, item) in root.items.iter().enumerate() {
                    println!("Item {}: {:?}", i, item);
                }
            }
            Err(error) => {
                println!("Parsing failed: {}", error);
            }
        }
    }
}
