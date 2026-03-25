export const storage = {
    /**
     * 存储数据到本地存储
     * @param key 存储键名
     * @param value 存储值
     */
    set<T>(key: string, value: T): void {
        try {
            localStorage.setItem(key, JSON.stringify(value));
        } catch (error) {
            console.error("Error saving to localStorage:", error);
        }
    },

    /**
     * 从本地存储获取数据
     * @param key 存储键名
     * @param defaultValue 默认值
     * @returns 存储的值或默认值
     */
    get<T>(key: string, defaultValue: T): T {
        try {
            const item = localStorage.getItem(key);
            return item ? JSON.parse(item) : defaultValue;
        } catch (error) {
            console.error("Error reading from localStorage:", error);
            return defaultValue;
        }
    },

    /**
     * 从本地存储删除数据
     * @param key 存储键名
     */
    remove(key: string): void {
        try {
            localStorage.removeItem(key);
        } catch (error) {
            console.error("Error removing from localStorage:", error);
        }
    },

    /**
     * 清除所有本地存储数据
     */
    clear(): void {
        try {
            localStorage.clear();
        } catch (error) {
            console.error("Error clearing localStorage:", error);
        }
    },
};
