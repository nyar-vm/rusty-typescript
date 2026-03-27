import { mkdir, readFile, writeFile, unlink, rename } from "node:fs/promises";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, "..");
const testsDir = join(rootDir, "compilers", "typescript", "tests");
const binDir = join(rootDir, "compilers", "typescript", "src", "bin");

async function ensureDir(dir) {
    try {
        await mkdir(dir, { recursive: true });
        console.log(`Created directory: ${dir}`);
    } catch (err) {
        if (err.code !== "EEXIST") throw err;
    }
}

async function safeUnlink(filePath) {
    try {
        await unlink(filePath);
        console.log(`Deleted: ${filePath}`);
    } catch (err) {
        if (err.code !== "ENOENT") {
            console.warn(`Warning: Could not delete ${filePath}: ${err.message}`);
        }
    }
}

async function safeReadFile(filePath) {
    try {
        return await readFile(filePath, "utf-8");
    } catch (err) {
        console.warn(`Warning: Could not read ${filePath}: ${err.message}`);
        return "";
    }
}

async function createTestFile(targetPath, content) {
    await writeFile(targetPath, content, "utf-8");
    console.log(`Created: ${targetPath}`);
}

async function safeRename(oldPath, newPath) {
    try {
        await rename(oldPath, newPath);
        console.log(`Renamed: ${oldPath} -> ${newPath}`);
    } catch (err) {
        console.warn(`Warning: Could not rename ${oldPath}: ${err.message}`);
    }
}

async function main() {
    console.log("Starting test file reorganization...\n");

    const newDirs = [
        join(testsDir, "parser"),
        join(testsDir, "runtime"),
        join(testsDir, "performance"),
        join(testsDir, "memory"),
        join(testsDir, "vm"),
        join(testsDir, "ffi"),
        join(testsDir, "webidl"),
    ];

    for (const dir of newDirs) {
        await ensureDir(dir);
    }

    console.log("\n--- Creating parser tests ---");

    const debugLexerParser = await safeReadFile(join(testsDir, "debug_lexer_parser.rs"));
    const lexerParserContent = `//! 词法分析和语法分析测试
//!
//! 测试 TypeScript 的词法分析和语法解析功能。

use typescript::test_lexer_parser;

#[test]
fn test_lexer_parser() {
    test_lexer_parser();
}
`;
    await createTestFile(join(testsDir, "parser", "lexer_parser.rs"), lexerParserContent);

    const moreSyntax = await safeReadFile(join(testsDir, "more_syntax.rs"));
    await createTestFile(join(testsDir, "parser", "syntax.rs"), moreSyntax);

    const newFeaturesTests = await safeReadFile(join(testsDir, "new_features_tests.rs"));
    await createTestFile(join(testsDir, "parser", "jsx.rs"), newFeaturesTests);

    const newFeatures = await safeReadFile(join(testsDir, "new_features.rs"));
    await createTestFile(join(testsDir, "parser", "decorators.rs"), newFeatures);

    const typescriptTests = await safeReadFile(join(testsDir, "typescript_tests.rs"));
    const typescriptFeaturesTest = await safeReadFile(
        join(testsDir, "typescript_features_test.rs"),
    );
    const typescriptContent = `//! TypeScript 特性解析测试
//!
//! 测试 TypeScript 特有语法的解析功能。

${typescriptTests}

${typescriptFeaturesTest}
`;
    await createTestFile(join(testsDir, "parser", "typescript.rs"), typescriptContent);

    console.log("\n--- Creating runtime tests ---");

    const runtimeTest = await safeReadFile(join(testsDir, "runtime_test.rs"));
    await createTestFile(join(testsDir, "runtime", "basic.rs"), runtimeTest);

    const codegenTest = await safeReadFile(join(testsDir, "codegen_test.rs"));
    await createTestFile(join(testsDir, "runtime", "codegen.rs"), codegenTest);

    const languageSpecTest = await safeReadFile(join(testsDir, "language_spec_test.rs"));
    await createTestFile(join(testsDir, "runtime", "language_spec.rs"), languageSpecTest);

    const crossPlatformTest = await safeReadFile(
        join(testsDir, "platform", "cross_platform_test.rs"),
    );
    const runtimeCrossPlatformContent = `//! 跨平台运行时测试
//!
//! 测试跨平台运行时功能。

${crossPlatformTest}
`;
    await createTestFile(
        join(testsDir, "runtime", "cross_platform.rs"),
        runtimeCrossPlatformContent,
    );

    console.log("\n--- Creating performance tests ---");

    const performanceBenchmark = await safeReadFile(join(testsDir, "performance_benchmark.rs"));
    await createTestFile(join(testsDir, "performance", "benchmark.rs"), performanceBenchmark);

    const jitTest = await safeReadFile(join(testsDir, "jit_test.rs"));
    const jitPerformance = await safeReadFile(join(testsDir, "jit_performance.rs"));
    const jitContent = `//! JIT 编译测试
//!
//! 测试 JIT 编译和优化功能。

${jitTest}

${jitPerformance}
`;
    await createTestFile(join(testsDir, "performance", "jit.rs"), jitContent);

    const testPerformance = await safeReadFile(join(binDir, "test-performance.rs"));
    const executionContent = `//! 执行性能测试
//!
//! 测试脚本执行性能。

${testPerformance.replace("fn main()", "#[test]\nfn test_execution_performance()")}
`;
    await createTestFile(join(testsDir, "performance", "execution.rs"), executionContent);

    const testPerformanceOptimized = await safeReadFile(
        join(binDir, "test-performance-optimized.rs"),
    );
    const optimizedContent = `//! 优化性能测试
//!
//! 测试优化后的性能效果。

${testPerformanceOptimized.replace("fn main()", "#[test]\nfn test_optimization_performance()")}
`;
    await createTestFile(join(testsDir, "performance", "optimized.rs"), optimizedContent);

    console.log("\n--- Creating memory tests ---");

    const gcTest = await safeReadFile(join(testsDir, "gc_test.rs"));
    await createTestFile(join(testsDir, "memory", "gc.rs"), gcTest);

    const memoryTest = await safeReadFile(join(testsDir, "memory_test.rs"));
    await createTestFile(join(testsDir, "memory", "pool.rs"), memoryTest);

    console.log("\n--- Creating vm tests ---");

    const vmTest = await safeReadFile(join(testsDir, "vm_test.rs"));
    await createTestFile(join(testsDir, "vm", "basic.rs"), vmTest);

    console.log("\n--- Creating ffi tests ---");

    const napiTest = await safeReadFile(join(testsDir, "napi_test.rs"));
    await createTestFile(join(testsDir, "ffi", "napi.rs"), napiTest);

    const napiModuleTest = await safeReadFile(join(testsDir, "napi_module_test.rs"));
    await createTestFile(join(testsDir, "ffi", "module.rs"), napiModuleTest);

    console.log("\n--- Creating platform tests ---");

    const platformTest = await safeReadFile(join(testsDir, "platform_test.rs"));
    await createTestFile(join(testsDir, "platform", "info.rs"), platformTest);

    const dylibTest = await safeReadFile(join(testsDir, "platform", "dylib_test.rs"));
    await createTestFile(join(testsDir, "platform", "dylib.rs"), dylibTest);

    console.log("\n--- Creating webidl tests ---");

    const webidlTest = await safeReadFile(join(testsDir, "webidl_test.rs"));
    await createTestFile(join(testsDir, "webidl", "basic.rs"), webidlTest);

    console.log("\n--- Deleting old test files ---");

    const filesToDelete = [
        join(testsDir, "debug_lexer_parser.rs"),
        join(testsDir, "more_syntax.rs"),
        join(testsDir, "new_features.rs"),
        join(testsDir, "new_features_tests.rs"),
        join(testsDir, "typescript_tests.rs"),
        join(testsDir, "typescript_features_test.rs"),
        join(testsDir, "runtime_test.rs"),
        join(testsDir, "codegen_test.rs"),
        join(testsDir, "language_spec_test.rs"),
        join(testsDir, "performance_benchmark.rs"),
        join(testsDir, "jit_test.rs"),
        join(testsDir, "jit_performance.rs"),
        join(testsDir, "gc_test.rs"),
        join(testsDir, "memory_test.rs"),
        join(testsDir, "vm_test.rs"),
        join(testsDir, "napi_test.rs"),
        join(testsDir, "napi_module_test.rs"),
        join(testsDir, "platform_test.rs"),
        join(testsDir, "webidl_test.rs"),
        join(testsDir, "platform", "cross_platform_test.rs"),
        join(testsDir, "platform", "dylib_test.rs"),
        join(binDir, "test-performance.rs"),
        join(binDir, "test-performance-optimized.rs"),
    ];

    for (const file of filesToDelete) {
        await safeUnlink(file);
    }

    console.log("\n=== Reorganization complete! ===");
}

main().catch(console.error);
