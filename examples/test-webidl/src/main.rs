use typescript_webidl::{convert_to_typescript, parse};

fn main() {
    let idl = r#"
        enum TestEnum {
            "value1",
            "value2"
        }
    "#;

    println!("WebIDL input:");
    println!("{}", idl);

    let result = parse(idl);
    println!("\nParse result:");
    match result {
        Ok(root) => {
            println!("Parsed successfully!");
            println!("Number of items: {}", root.items.len());
            for (i, item) in root.items.iter().enumerate() {
                println!("Item {}: {:?}", i, item);
            }

            let typescript = convert_to_typescript(&root);
            println!("\nConvert result:");
            println!("Length: {}", typescript.len());
            println!("Content:\n{}", typescript);
        }
        Err(error) => {
            println!("Parse error: {}", error);
        }
    }
}
