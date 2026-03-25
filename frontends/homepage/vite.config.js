import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { VitePWA } from "vite-plugin-pwa";
import { resolve } from "path";
export default defineConfig({
    plugins: [
        vue(),
        VitePWA({
            registerType: "autoUpdate",
            workbox: {
                globPatterns: ["**/*.{js,css,html,ico,png,svg}"],
                maximumFileSizeToCacheInBytes: 10 * 1024 * 1024, // 10 MB
                runtimeCaching: [
                    {
                        urlPattern: /^https:\/\/cdnjs\.cloudflare\.com\//i,
                        handler: "CacheFirst",
                        options: {
                            cacheName: "cdn-cache",
                            expiration: {
                                maxEntries: 50,
                                maxAgeSeconds: 60 * 60 * 24 * 30, // 30 days
                            },
                        },
                    },
                ],
            },
            includeAssets: ["favicon.ico", "robots.txt"],
            manifest: {
                name: "Rusty TypeScript",
                short_name: "RustyTS",
                description: "Rust 实现的 TypeScript 编译器和运行时",
                theme_color: "#3b82f6",
                icons: [
                    {
                        src: "favicon.ico",
                        sizes: "64x64 32x32 24x24 16x16",
                        type: "image/x-icon",
                    },
                ],
            },
        }),
    ],
    resolve: {
        alias: {
            "@": resolve(__dirname, "src"),
        },
    },
    server: {
        fs: {
            allow: ["../.."],
        },
    },
    build: {
        target: "esnext",
        rollupOptions: {
            output: {
                manualChunks: {
                    vendor: ["vue"],
                    element: ["element-plus"],
                },
                entryFileNames: "assets/[name].[hash].js",
                chunkFileNames: "assets/[name].[hash].js",
                assetFileNames: "assets/[name].[hash].[ext]",
            },
        },
        minify: "terser",
        terserOptions: {
            compress: {
                drop_console: true,
                drop_debugger: true,
            },
        },
        cssCodeSplit: true,
        sourcemap: false,
    },
    optimizeDeps: {
        include: ["vue", "element-plus"],
        exclude: [],
    },
});
