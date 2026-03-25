/// 调试测试
use typescript_webidl::debug::debug_parse;

#[test]
fn test_debug_parse() {
    println!("Running debug parse test...");
    debug_parse();
    println!("Debug parse test completed.");
}
