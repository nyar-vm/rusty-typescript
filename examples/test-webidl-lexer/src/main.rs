use oak_core::{LexerCache, lexer::LexOutput, source::SourceText};
use oak_idl::{IdlLanguage, IdlLexer};
use std::sync::Arc;

fn main() {
    let idl = r#"
        interface TestInterface {
            attribute string name;
            attribute long age;
            void doSomething();
            string getName();
        }
    "#;

    println!("WebIDL input:");
    println!("{}", idl);

    let language = IdlLanguage::default();
    let lexer = IdlLexer::new(&language);
    let source = SourceText::new(idl.to_string());
    let mut cache = oak_core::lexer::session::LexSession::default();
    let lex_result = lexer.lex(&source, &[], &mut cache);

    println!("\nLex result:");
    for token in lex_result.tokens {
        println!("Token: {:?}, Text: {:?}", token.kind, source.get_text_in(token.range));
    }
}
