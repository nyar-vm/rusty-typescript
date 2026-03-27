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
                maximumFileSizeToCacheInBytes: 10 * 1024 * 1024,
                runtimeCaching: [
                    {
                        urlPattern: /^https:\/\/cdnjs\.cloudflare\.com\//i,
                        handler: "CacheFirst",
                        options: {
                            cacheName: "cdn-cache",
                            expiration: {
                                maxEntries: 50,
                                maxAgeSeconds: 60 * 60 * 24 * 30,
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
                manualChunks: function (id) {
                    if (id.includes("node_modules")) {
                        if (id.includes("monaco-editor")) {
                            return "monaco-editor";
                        }
                        if (id.includes("vue") || id.includes("@vue")) {
                            return "vue-vendor";
                        }
                        if (id.includes("element-plus")) {
                            return "element-plus";
                        }
                        if (id.includes("@nyar/typescript")) {
                            return "nyar-typescript";
                        }
                        if (id.includes("@element-plus/icons-vue")) {
                            return "element-icons";
                        }
                        return "vendor";
                    }
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
                pure_funcs: ["console.log", "console.info", "console.debug"],
                passes: 2,
                ecma: 2020,
                comparisons: false,
                inline: 2,
                collapse_vars: true,
                reduce_vars: true,
                booleans: true,
                loops: true,
                unused: true,
                dead_code: true,
            },
            format: {
                comments: false,
            },
            mangle: {
                safari10: true,
                properties: {
                    regex: /^_/,
                },
            },
        },
        cssCodeSplit: true,
        sourcemap: false,
        chunkSizeWarningLimit: 1000,
        reportCompressedSize: true,
    },
    optimizeDeps: {
        include: ["vue", "element-plus"],
        exclude: [],
        esbuildOptions: {
            target: "esnext",
        },
    },
});
