#!/usr/bin/env node

/**
 * 跨平台测试脚本
 *
 * 用于在本地运行跨平台测试，提供统一的测试报告
 * 支持在 Windows、Linux、macOS 上运行
 */

import { execSync } from "child_process";
import { existsSync, readdirSync, writeFileSync } from "fs";
import { join } from "path";

// 项目根目录
const projectRoot = join(process.cwd(), "..");

// 测试结果存储
const testResults = {
    platform: process.platform,
    arch: process.arch,
    tests: {},
    summary: {
        total: 0,
        passed: 0,
        failed: 0,
        duration: 0,
    },
};

/**
 * 运行测试命令
 * @param {string} command - 测试命令
 * @param {string} testName - 测试名称
 * @returns {Object} 测试结果
 */
function runTest(command, testName) {
    console.log(`\n=== 运行测试: ${testName} ===`);
    console.log(`命令: ${command}`);

    const startTime = Date.now();
    let output = "";
    let success = false;

    try {
        output = execSync(command, {
            cwd: projectRoot,
            stdio: "inherit",
            encoding: "utf8",
        });
        success = true;
        console.log(`✅ ${testName} 测试通过`);
    } catch (error) {
        output = error.stdout || error.message;
        success = false;
        console.log(`❌ ${testName} 测试失败`);
    }

    const duration = Date.now() - startTime;

    return {
        success,
        duration,
        output,
    };
}

/**
 * 运行所有测试
 */
function runAllTests() {
    console.log("开始跨平台测试...");
    console.log(`平台: ${process.platform}`);
    console.log(`架构: ${process.arch}`);

    // 运行核心测试
    const coreTests = runTest("cargo test --all-targets --all-features", "核心测试");
    testResults.tests.core = coreTests;

    // 运行平台特定测试
    const platformTests = runTest("cargo test --test platform_test", "平台测试");
    testResults.tests.platform = platformTests;

    // 运行 FFI/NAPI 测试
    const ffiTests = runTest("cargo test --test napi_test", "FFI/NAPI 测试");
    testResults.tests.ffi = ffiTests;

    // 运行运行时测试
    const runtimeTests = runTest("cargo test --test runtime_test", "运行时测试");
    testResults.tests.runtime = runtimeTests;

    // 计算测试结果
    calculateSummary();

    // 生成测试报告
    generateReport();

    console.log("\n=== 测试完成 ===");
    console.log(`总测试数: ${testResults.summary.total}`);
    console.log(`通过: ${testResults.summary.passed}`);
    console.log(`失败: ${testResults.summary.failed}`);
    console.log(`总耗时: ${testResults.summary.duration}ms`);

    if (testResults.summary.failed > 0) {
        console.log("❌ 有测试失败，请检查输出");
        process.exit(1);
    } else {
        console.log("✅ 所有测试通过");
        process.exit(0);
    }
}

/**
 * 计算测试摘要
 */
function calculateSummary() {
    let total = 0;
    let passed = 0;
    let failed = 0;
    let duration = 0;

    Object.values(testResults.tests).forEach((result) => {
        total++;
        duration += result.duration;
        if (result.success) {
            passed++;
        } else {
            failed++;
        }
    });

    testResults.summary = {
        total,
        passed,
        failed,
        duration,
    };
}

/**
 * 生成测试报告
 */
function generateReport() {
    const reportDir = join(projectRoot, "target", "test-reports");

    // 创建报告目录
    if (!existsSync(reportDir)) {
        execSync(`mkdir -p ${reportDir}`, { cwd: projectRoot });
    }

    // 生成 JSON 报告
    const jsonReportPath = join(reportDir, `test-results-${process.platform}-${process.arch}.json`);
    writeFileSync(jsonReportPath, JSON.stringify(testResults, null, 2));

    // 生成文本报告
    const textReportPath = join(reportDir, `test-results-${process.platform}-${process.arch}.txt`);
    const textReport = `
跨平台测试报告
=============

平台: ${process.platform}
架构: ${process.arch}

测试结果:
----------

核心测试: ${testResults.tests.core.success ? "通过" : "失败"} (${testResults.tests.core.duration}ms)
平台测试: ${testResults.tests.platform.success ? "通过" : "失败"} (${testResults.tests.platform.duration}ms)
FFI/NAPI 测试: ${testResults.tests.ffi.success ? "通过" : "失败"} (${testResults.tests.ffi.duration}ms)
运行时测试: ${testResults.tests.runtime.success ? "通过" : "失败"} (${testResults.tests.runtime.duration}ms)

摘要:
-----
总测试数: ${testResults.summary.total}
通过: ${testResults.summary.passed}
失败: ${testResults.summary.failed}
总耗时: ${testResults.summary.duration}ms

${testResults.summary.failed > 0 ? "❌ 有测试失败" : "✅ 所有测试通过"}
`;

    writeFileSync(textReportPath, textReport);

    console.log(`\n测试报告已生成:`);
    console.log(`- JSON 报告: ${jsonReportPath}`);
    console.log(`- 文本报告: ${textReportPath}`);
}

/**
 * 主函数
 */
function main() {
    // 检查是否在项目根目录
    if (!existsSync(join(projectRoot, "Cargo.toml"))) {
        console.error("错误: 请在项目根目录运行此脚本");
        process.exit(1);
    }

    // 检查 Rust 是否安装
    try {
        execSync("cargo --version", { stdio: "ignore" });
    } catch (error) {
        console.error("错误: Rust 未安装或不在 PATH 中");
        process.exit(1);
    }

    // 运行测试
    runAllTests();
}

// 执行主函数
if (import.meta.url === `file://${process.argv[1]}`) {
    main();
}

export default {
    runAllTests,
    runTest,
    calculateSummary,
    generateReport,
};
