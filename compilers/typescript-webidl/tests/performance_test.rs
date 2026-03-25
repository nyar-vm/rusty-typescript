/// WebIDL 解析和转换性能测试
use std::time::Instant;
use typescript_webidl::{convert_to_typescript, parse};

#[test]
fn test_parse_performance() {
    // 测试解析性能
    let complex_idl = r#"
        enum MediaType {
            "audio",
            "video",
            "image"
        }
        
        dictionary MediaOptions {
            MediaType type;
            boolean autoplay;
            sequence<string> sources;
        }
        
        interface MediaPlayer {
            attribute MediaOptions options;
            void play();
            void pause();
            Promise<void> load(string url);
            (string or Error) getError();
        }
        
        interface MediaRecorder {
            attribute MediaStream stream;
            void start();
            void stop();
            Promise<Blob> capture();
        }
        
        dictionary CaptureOptions {
            long width;
            long height;
            double frameRate;
        }
        
        interface VideoCapture {
            attribute CaptureOptions options;
            MediaStream getStream();
            Promise<void> start();
            void stop();
        }
    "#;

    // 测量解析时间
    let start = Instant::now();
    let result = parse(complex_idl);
    let duration = start.elapsed();

    println!("解析时间: {:?}", duration);
    assert!(result.is_ok(), "解析应该成功");

    // 测量转换时间
    if let Ok(root) = result {
        let start = Instant::now();
        let typescript = convert_to_typescript(&root);
        let duration = start.elapsed();

        println!("转换时间: {:?}", duration);
        println!("转换结果长度: {}", typescript.len());
    }
}

#[test]
fn test_parse_large_webidl() {
    // 测试解析大型 WebIDL
    let mut large_idl = String::new();

    // 生成大型 WebIDL
    for i in 0..100 {
        large_idl.push_str(&format!(
            r#"
            interface TestInterface{} {{
                attribute string name{};
                attribute long id{};
                void method{}(string param{});
                string getValue{}();
            }}
            "#,
            i, i, i, i, i, i
        ));
    }

    // 测量解析时间
    let start = Instant::now();
    let result = parse(&large_idl);
    let duration = start.elapsed();

    println!("大型 WebIDL 解析时间: {:?}", duration);
    assert!(result.is_ok(), "解析大型 WebIDL 应该成功");

    // 测量转换时间
    if let Ok(root) = result {
        let start = Instant::now();
        let typescript = convert_to_typescript(&root);
        let duration = start.elapsed();

        println!("大型 WebIDL 转换时间: {:?}", duration);
        println!("转换结果长度: {}", typescript.len());
    }
}
