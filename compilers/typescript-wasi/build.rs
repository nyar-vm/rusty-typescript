fn main() {
    // 告诉 Cargo 重新生成绑定当 WIT 文件变化时
    println!("cargo:rerun-if-changed=typescript.wit");
}
