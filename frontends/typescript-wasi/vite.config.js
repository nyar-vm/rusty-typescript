import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
    build: {
        target: "esnext",
        outDir: "dist",
        minify: "terser",
        terserOptions: {
            compress: {
                drop_console: true,
                drop_debugger: true,
            },
            mangle: {
                toplevel: true,
            },
        },
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
            entry: "./src/index.ts",
            name: "RustyTypeScript",
            formats: ["es"],
        },
    },
    optimizeDeps: {
        esbuildOptions: {
            target: "esnext",
        },
    },
});
