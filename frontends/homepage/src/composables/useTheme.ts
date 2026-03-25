import { ref, computed, watch, onMounted } from "vue";
import { storage } from "../utils/storage";
import { ThemeMode, ThemeConfig } from "../types/theme";
import { lightTheme, darkTheme } from "../config/themes";

const STORAGE_KEY = "rusty-typescript-theme";

const defaultConfig: ThemeConfig = {
    mode: "system",
    accentColor: "#3b82f6",
    fontSize: 14,
    fontFamily: "Inter, sans-serif",
};

export function useTheme() {
    const config = ref<ThemeConfig>(storage.get(STORAGE_KEY, defaultConfig));
    const isSystemDark = ref(false);

    // 计算当前实际使用的主题
    const currentTheme = computed(() => {
        let mode: "light" | "dark";

        if (config.value.mode === "system") {
            mode = isSystemDark.value ? "dark" : "light";
        } else {
            mode = config.value.mode;
        }

        return mode === "dark" ? darkTheme : lightTheme;
    });

    // 计算当前是否为暗色模式
    const isDark = computed(() => {
        return currentTheme.value.mode === "dark";
    });

    // 应用主题到文档
    const applyTheme = () => {
        const theme = currentTheme.value;

        // 移除旧的主题类
        document.documentElement.classList.remove("light", "dark");
        document.documentElement.classList.add(theme.mode);

        // 设置CSS变量
        Object.entries(theme.colors).forEach(([key, value]) => {
            document.documentElement.style.setProperty(`--color-${key}`, value);
        });

        // 设置字体大小和字体
        document.documentElement.style.setProperty("--font-size", `${config.value.fontSize}px`);
        document.documentElement.style.setProperty("--font-family", config.value.fontFamily);
    };

    // 切换主题模式
    const setThemeMode = (mode: ThemeMode) => {
        config.value.mode = mode;
        saveConfig();
    };

    // 设置强调色
    const setAccentColor = (color: string) => {
        config.value.accentColor = color;
        saveConfig();
    };

    // 设置字体大小
    const setFontSize = (size: number) => {
        config.value.fontSize = size;
        saveConfig();
    };

    // 设置字体
    const setFontFamily = (font: string) => {
        config.value.fontFamily = font;
        saveConfig();
    };

    // 保存配置到本地存储
    const saveConfig = () => {
        storage.set(STORAGE_KEY, config.value);
    };

    // 监听系统主题变化
    const handleSystemThemeChange = (e: MediaQueryListEvent) => {
        isSystemDark.value = e.matches;
    };

    // 初始化
    onMounted(() => {
        // 检测系统主题
        const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
        isSystemDark.value = mediaQuery.matches;
        mediaQuery.addEventListener("change", handleSystemThemeChange);

        // 应用主题
        applyTheme();
    });

    // 监听主题变化并应用
    watch(
        [config, isSystemDark],
        () => {
            applyTheme();
        },
        { deep: true },
    );

    return {
        config,
        currentTheme,
        isDark,
        setThemeMode,
        setAccentColor,
        setFontSize,
        setFontFamily,
    };
}
