// TypeScript 语法特性测试

// 变量声明与类型注解
let x: number = 10;
const y: string = "hello";
var z: boolean = true;

// 接口定义
interface Person {
    name: string;
    age: number;
    address?: string;
}

// 类定义
class Employee implements Person {
    name: string;
    age: number;
    address?: string;
    private salary: number;

    constructor(name: string, age: number, salary: number) {
        this.name = name;
        this.age = age;
        this.salary = salary;
    }

    getSalary(): number {
        return this.salary;
    }
}

// 函数定义与类型注解
function add(a: number, b: number): number {
    return a + b;
}

// 箭头函数
const multiply = (a: number, b: number): number => a * b;

// 泛型
function identity<T>(value: T): T {
    return value;
}

// 联合类型
let result: number | string = 42;
result = "forty-two";

// 交叉类型
interface A {
    a: number;
}

interface B {
    b: string;
}

let ab: A & B = { a: 1, b: "hello" };

// 可选链
const user = {
    profile: {
        name: "John",
    },
};

const userName = user?.profile?.name;

// 空值合并
const defaultValue = user?.profile?.age ?? 18;

// 模板字符串
const greeting = `Hello, ${user?.profile?.name}!`;

// 异步函数
async function fetchData(): Promise<string> {
    return "data";
}

// 枚举
enum Color {
    Red,
    Green,
    Blue,
}

const color: Color = Color.Red;

// 命名空间
namespace MyNamespace {
    export const value = 42;
}

// 模块导入导出
export interface ExportedInterface {
    value: number;
}

export function exportedFunction(): void {
    console.log("Exported function");
}

import { ExportedInterface, exportedFunction } from "./typescript_features";
