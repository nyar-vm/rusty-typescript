import { defineConfig } from "vite";

export default defineConfig({
    build: {
        target: "esnext",
        outDir: "dist",
        rollupOptions: {
            output: {
                entryFileNames: "[name].js",
                chunkFileNames: "[name].js",
                assetFileNames: "[name].[ext]",
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
