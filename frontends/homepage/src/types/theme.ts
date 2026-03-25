export type ThemeMode = "light" | "dark" | "system";

export interface ThemeConfig {
    mode: ThemeMode;
    accentColor: string;
    fontSize: number;
    fontFamily: string;
}

export interface Theme {
    name: string;
    mode: "light" | "dark";
    colors: {
        primary: string;
        secondary: string;
        background: string;
        surface: string;
        text: string;
        textSecondary: string;
        border: string;
        accent: string;
        success: string;
        warning: string;
        error: string;
        info: string;
    };
}
