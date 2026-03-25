use typescript_webidl::parse;

#[test]
fn test_parse_function() {
    let webidl = r#"
        interface TestInterface {
            void test();
        };
    "#;

    println!("Testing typescript_webidl::parse...");
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
