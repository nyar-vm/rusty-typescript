import pako from "pako";

/**
 * 将代码压缩并编码为 URL 安全的 Base64 字符串
 * @param code 要编码的代码字符串
 * @returns 编码后的字符串，可直接用于 URL 参数
 */
export function encodeCodeToUrl(code: string): string {
    const encoder = new TextEncoder();
    const uint8Array = encoder.encode(code);
    const compressed = pako.deflate(uint8Array);
    let binary = "";
    for (let i = 0; i < compressed.length; i++) {
        binary += String.fromCharCode(compressed[i]);
    }
    const base64 = btoa(binary);
    return base64.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/**
 * 从 URL 安全的 Base64 字符串解码并解压缩代码
 * @param encoded 编码后的字符串
 * @returns 解码后的原始代码字符串
 */
export function decodeCodeFromUrl(encoded: string): string {
    let base64 = encoded.replace(/-/g, "+").replace(/_/g, "/");
    while (base64.length % 4) {
        base64 += "=";
    }
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    const decompressed = pako.inflate(bytes);
    const decoder = new TextDecoder();
    return decoder.decode(decompressed);
}
