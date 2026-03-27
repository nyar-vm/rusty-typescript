# JIT 编译模块

提供即时编译功能，将热点代码编译为机器码，提高执行性能。

## 模块结构

- [`compiler`] - JIT 编译器核心，负责热点检测和函数编译
- [`optimizer`] - JIT 优化器，提供指令级别的优化
- [`executor`] - JIT 执行器，执行 JIT 编译后的代码

## 主要功能

- 自适应热点阈值算法：根据函数复杂度动态调整编译阈值
- 优先级队列：按执行时间和调用次数排序热点函数
- 编译状态跟踪：跟踪函数的编译状态和统计信息
- 事件回调机制：支持编译完成通知

## 示例

```ignore
use rusty_typescript::jit::{JITCompiler, JITEventCallback, JITEvent};

// 创建 JIT 编译器
let mut compiler = JITCompiler::new();

// 注册事件回调
struct MyCallback;
impl JITEventCallback for MyCallback {
    fn on_event(&self, event: &JITEvent) {
        println!("JIT Event: {:?}", event);
    }
}
compiler.register_callback(std::rc::Rc::new(MyCallback));

// 检查是否需要编译
if compiler.should_compile("hot_function") {
    // 编译函数
    compiler.compile_function("hot_function", &instructions);
}
```
