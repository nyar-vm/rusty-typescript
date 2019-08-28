<template>
  <div class="typescript-playground">
    <h2>TypeScript Playground</h2>
    <div class="playground-container">
      <div class="code-editor">
        <div ref="editorContainer" class="editor-container"></div>
      </div>
      <div class="playground-controls">
        <button @click="runCode" class="run-button">Run</button>
        <button @click="resetCode" class="reset-button">Reset</button>
      </div>
      <div class="output-container">
        <h3>Output:</h3>
        <div class="output" v-html="output"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import editorConfig from "../config/editorConfig.json";

const editorContainer = ref<HTMLElement | null>(null);
let editor: any = null;
const output = ref("");

const defaultCode = `console.log('Hello, TypeScript!');

function greet(name: string): string {
  return "Hello, " + name + "!";
}

console.log(greet('World'));`;

const runCode = async () => {
    if (editor) {
        const code = editor.getValue();
        // In the future, this will call the TypeScript WASI module
        output.value = `Executing TypeScript code:
${code}

Output:
Hello, TypeScript!
Hello, World!`;
    }
};

const resetCode = () => {
    if (editor) {
        editor.setValue(defaultCode);
    }
    output.value = "";
};

onMounted(() => {
    // Load Monaco Editor from CDN
    if (typeof window !== "undefined") {
        const script = document.createElement("script");
        script.src =
            "https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs/loader.min.js";
        script.onload = () => {
            // @ts-ignore
            window.require.config({
                paths: {
                    vs: "https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.45.0/min/vs",
                },
            });
            // @ts-ignore
            window.require(["vs/editor/editor.main"], () => {
                // Define One Dark Pro theme from imported config
                // @ts-ignore
                monaco.editor.defineTheme("one-dark-pro", editorConfig.theme);
                // @ts-ignore
                editor = monaco.editor.create(editorContainer.value, {
                    value: defaultCode,
                    ...editorConfig.editorOptions,
                });
            });
        };
        document.head.appendChild(script);
    }
});

onUnmounted(() => {
    if (editor) {
        editor.dispose();
    }
});
</script>

<style scoped>
.typescript-playground {
  width: 100%;
  max-width: 1000px;
  margin: 0 auto;
  padding: 20px;
  background-color: #f5f5f5;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.typescript-playground h2 {
  text-align: center;
  margin-bottom: 20px;
  color: #333;
}

.playground-container {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.code-editor {
  width: 100%;
}

.editor-container {
  width: 100%;
  min-height: 300px;
  border-radius: 4px;
  overflow: hidden;
}

.playground-controls {
  display: flex;
  gap: 10px;
}

.run-button, .reset-button {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: bold;
}

.run-button {
  background-color: #4CAF50;
  color: white;
}

.run-button:hover {
  background-color: #45a049;
}

.reset-button {
  background-color: #f44336;
  color: white;
}

.reset-button:hover {
  background-color: #da190b;
}

.output-container {
  width: 100%;
  background-color: white;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 15px;
}

.output-container h3 {
  margin-top: 0;
  margin-bottom: 10px;
  color: #333;
}

.output {
  font-family: 'Courier New', Courier, monospace;
  font-size: 14px;
  white-space: pre-wrap;
  line-height: 1.5;
}
</style>
