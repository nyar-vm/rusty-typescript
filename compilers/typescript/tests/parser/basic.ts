// 基本变量声明
let x: number = 10;
let y: string = "hello";

// 函数定义
function add(a: number, b: number): number {
    return a + b;
}

// 函数调用
const result = add(x, y);

// 类定义
class Person {
    name: string;
    age: number;

    constructor(name: string, age: number) {
        this.name = name;
        this.age = age;
    }

    greet(): string {
        return `Hello, my name is ${this.name}`;
    }
}

// 接口定义
interface Animal {
    name: string;
    makeSound(): void;
}

// 实现接口的类
class Dog implements Animal {
    name: string;

    constructor(name: string) {
        this.name = name;
    }

    makeSound(): void {
        console.log("Woof!");
    }
}

// 使用类和接口
const person = new Person("Alice", 30);
const dog = new Dog("Buddy");

console.log(person.greet());
dog.makeSound();
