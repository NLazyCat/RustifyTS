// More complex TypeScript features
interface User {
    id: number;
    name: string;
    email?: string;
}

type Point = {
    x: number;
    y: number;
};

class Rectangle {
    private width: number;
    private height: number;

    constructor(width: number, height: number) {
        this.width = width;
        this.height = height;
    }

    getArea(): number {
        return this.width * this.height;
    }
}

// Arrow function
const multiply = (a: number, b: number): number => a * b;

// Array and object literals
const numbers: number[] = [1, 2, 3, 4, 5];
const person = {
    name: "Bob",
    age: 30,
    isActive: true
};

// Union type
type Result = Success | Failure;

interface Success {
    type: "success";
    data: string;
}

interface Failure {
    type: "failure";
    error: string;
}
