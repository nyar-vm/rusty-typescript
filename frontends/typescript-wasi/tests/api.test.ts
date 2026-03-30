import { describe, it, expect } from "vitest";
import {
    RustyTypeScript,
    compileTypeScript,
    executeTypeScript,
    getCompilationErrors,
} from "../src/api";

describe("RustyTypeScript", () => {
    it("should create a new instance", () => {
        const ts = new RustyTypeScript();
        expect(ts).toBeInstanceOf(RustyTypeScript);
    });

    it("should have load method", () => {
        const ts = new RustyTypeScript();
        expect(typeof ts.load).toBe("function");
    });

    it("should have compileTypeScript method", () => {
        const ts = new RustyTypeScript();
        expect(typeof ts.compileTypeScript).toBe("function");
    });

    it("should have executeTypeScript method", () => {
        const ts = new RustyTypeScript();
        expect(typeof ts.executeTypeScript).toBe("function");
    });

    it("should have getCompilationErrors method", () => {
        const ts = new RustyTypeScript();
        expect(typeof ts.getCompilationErrors).toBe("function");
    });

    it("should have reset method", () => {
        const ts = new RustyTypeScript();
        expect(typeof ts.reset).toBe("function");
    });
});

describe("API functions", () => {
    it("should have compileTypeScript function", () => {
        expect(typeof compileTypeScript).toBe("function");
    });

    it("should have executeTypeScript function", () => {
        expect(typeof executeTypeScript).toBe("function");
    });

    it("should have getCompilationErrors function", () => {
        expect(typeof getCompilationErrors).toBe("function");
    });
});
