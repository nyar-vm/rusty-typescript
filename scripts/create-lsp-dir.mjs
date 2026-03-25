import { mkdirSync, existsSync } from "fs";
import { join } from "path";

const baseDir = join("compilers", "typescript-lsp");
const lspDir = join(baseDir, "src", "lsp");
const mcpDir = join(baseDir, "src", "mcp");

// 创建目录结构
[baseDir, join(baseDir, "src"), lspDir, mcpDir].forEach((dir) => {
    if (!existsSync(dir)) {
        mkdirSync(dir, { recursive: true });
        console.log(`Created directory: ${dir}`);
    } else {
        console.log(`Directory already exists: ${dir}`);
    }
});

console.log("Directory structure created successfully!");
