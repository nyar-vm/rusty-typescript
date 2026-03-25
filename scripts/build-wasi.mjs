import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.join(__dirname, "..");
const isWindows = process.platform === "win32";

function runCommand(command, cwd = rootDir) {
    console.log(`Executing: ${command} in ${cwd}`);
    execSync(command, { stdio: "inherit", cwd });
}

async function buildTypeScriptWasi() {
    console.log("Starting TypeScript WASI build process...");

    try {
        // 步骤 1: 确保 wasm32-wasip2 目标已安装
        console.log("\nStep 1: Checking for wasm32-wasip2 target...");
        try {
            runCommand("rustup target add wasm32-wasip2");
        } catch (error) {
            console.log(
                "\x1b[33m⚠ Target wasm32-wasip2 already installed or error occurred\x1b[0m",
            );
        }

        // 步骤 2: 构建 typescript-wasi 到 wasm32-wasip2 目标
        console.log("\nStep 2: Building typescript-wasi with wasm32-wasip2 target...");
        runCommand("cargo build --package typescript-wasi --target wasm32-wasip2 --release");

        // 步骤 3: 跳过 jco 安装，直接使用 npx
        console.log("\nStep 3: Skipping jco installation, will use npx...");

        // 步骤 4: 创建输出目录
        const outputDir = path.join(rootDir, "frontends", "typescript-wasi", "lib");
        console.log(`\nStep 4: Creating output directory: ${outputDir}...`);
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }

        // 步骤 5: 使用 jco 转译 WASM 模块
        console.log("\nStep 5: Translating WASM module with jco...");
        const wasmPath = path.join(
            rootDir,
            "target",
            "wasm32-wasip2",
            "release",
            "typescript_wasi.wasm",
        );
        runCommand(`npx jco transpile ${wasmPath} --out-dir ${outputDir} --name typescriptWasi`);

        // 步骤 6: 复制 WASM 文件到前端目录
        console.log("\nStep 6: Copying WASM file to frontend directory...");
        const frontendWasmPath = path.join(
            rootDir,
            "frontends",
            "typescript-wasi",
            "src",
            "typescript_wasi.wasm",
        );
        fs.copyFileSync(wasmPath, frontendWasmPath);

        console.log("\n\x1b[32m=== TypeScript WASI Build Process Complete ===\x1b[0m");
        console.log(`WASM module built and transpiled to: ${outputDir}`);
        console.log(`WASM file copied to: ${frontendWasmPath}`);
    } catch (error) {
        console.error("\n\x1b[31mError during TypeScript WASI build:\x1b[0m", error);
        process.exit(1);
    }
}

buildTypeScriptWasi();
