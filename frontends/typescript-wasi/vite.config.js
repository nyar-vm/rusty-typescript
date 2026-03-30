import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
    build: {
        target: "esnext",
        outDir: "dist",
        minify: true,
        sourcemap: false,
        rollupOptions: {
            output: {
                entryFileNames: "[name].js",
                chunkFileNames: "[name].js",
                assetFileNames: "[name].[ext]",
                minifyInternalExports: true,
            },
        },
        lib: {
            entry: resolve(__dirname, "src", "index.ts"),
            name: "RustyTypeScript",
            formats: ["es"],
        },
    },
    optimizeDeps: {
        esbuildOptions: {
            target: "esnext",
        },
    },
    worker: {
        format: 'es',
    },
    resolve: {
        alias: {
            '@': resolve(__dirname, 'src'),
        },
    },
});
