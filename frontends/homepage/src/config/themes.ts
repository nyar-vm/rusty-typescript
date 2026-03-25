import { Theme } from "../types/theme";

export const lightTheme: Theme = {
    name: "light",
    mode: "light",
    colors: {
        primary: "#3b82f6",
        secondary: "#6366f1",
        background: "#f8fafc",
        surface: "#ffffff",
        text: "#1e293b",
        textSecondary: "#64748b",
        border: "#e2e8f0",
        accent: "#8b5cf6",
        success: "#10b981",
        warning: "#f59e0b",
        error: "#ef4444",
        info: "#3b82f6",
    },
};

export const darkTheme: Theme = {
    name: "dark",
    mode: "dark",
    colors: {
        primary: "#60a5fa",
        secondary: "#818cf8",
        background: "#0f172a",
        surface: "#1e293b",
        text: "#f1f5f9",
        textSecondary: "#94a3b8",
        border: "#334155",
        accent: "#a78bfa",
        success: "#34d399",
        warning: "#fbbf24",
        error: "#f87171",
        info: "#60a5fa",
    },
};

export const themes = {
    light: lightTheme,
    dark: darkTheme,
};
