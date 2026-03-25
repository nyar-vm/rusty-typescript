<template>
  <div class="markdown-viewer prose prose-slate max-w-none">
    <div ref="contentRef" v-html="renderedContent"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { marked } from "marked";
import { createHighlighter } from "shiki";

interface Props {
    content: string;
}

const props = defineProps<Props>();
const contentRef = ref<HTMLElement | null>(null);
let highlighter: any = null;

onMounted(async () => {
    highlighter = await createHighlighter({
        themes: ["one-dark-pro"],
        langs: ["rust", "typescript", "javascript", "json", "bash", "ruby"],
    });
    highlightCodeBlocks();
    setupCodeExamples();
});

watch(
    () => props.content,
    () => {
        setTimeout(() => {
            highlightCodeBlocks();
            setupCodeExamples();
        }, 0);
    },
);

const highlightCodeBlocks = async () => {
    if (!highlighter || !contentRef.value) return;

    const codeBlocks = contentRef.value.querySelectorAll("pre code");
    for (const codeBlock of codeBlocks) {
        const code = codeBlock.textContent || "";
        const lang = codeBlock.className.replace("language-", "");

        try {
            const highlighted = highlighter.codeToHtml(code, {
                lang: lang || "text",
                theme: "one-dark-pro",
            });
            const pre = codeBlock.parentElement;
            if (pre) {
                pre.innerHTML = highlighted;
            }
        } catch (error) {
            console.error("Code highlighting error:", error);
        }
    }
};

const setupCodeExamples = () => {
    if (!contentRef.value) return;

    // 查找所有带有 data-example 属性的代码块
    const exampleBlocks = contentRef.value.querySelectorAll('[data-example="true"]');
    for (const block of exampleBlocks) {
        const code = block.textContent || "";
        const pre = block.parentElement;
        if (pre) {
            // 创建代码示例容器
            const container = document.createElement("div");
            container.className =
                "code-example-container rounded-lg overflow-hidden border border-blue-200";

            // 创建代码展示部分
            const codeSection = document.createElement("div");
            codeSection.className = "bg-gray-900 p-4";
            codeSection.innerHTML = block.outerHTML;

            // 创建运行按钮
            const runButton = document.createElement("button");
            runButton.className =
                "mt-2 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors";
            runButton.textContent = "运行示例";
            runButton.addEventListener("click", () => runCodeExample(code));

            // 创建结果显示部分
            const resultSection = document.createElement("div");
            resultSection.className = "bg-white p-4 border-t border-blue-200";
            resultSection.innerHTML = '<p class="text-slate-600">点击"运行示例"查看结果</p>';

            // 组装容器
            container.appendChild(codeSection);
            container.appendChild(runButton);
            container.appendChild(resultSection);

            // 替换原来的代码块
            pre.replaceWith(container);
        }
    }
};

const runCodeExample = (code: string) => {
    try {
        // 模拟代码运行
        console.log("Running code:", code);
        // 实际项目中，这里应该使用 TypeScript 编译器运行代码
        // 这里只是模拟结果
        const result =
            "代码运行成功！\n\n这是模拟的运行结果。\n在实际项目中，这里会显示真实的代码执行结果。";

        // 找到对应的结果显示区域并更新
        const resultSections = contentRef.value?.querySelectorAll(
            ".code-example-container .bg-white",
        );
        if (resultSections && resultSections.length > 0) {
            const lastResultSection = resultSections[resultSections.length - 1];
            lastResultSection.innerHTML = `<pre class="bg-gray-50 p-3 rounded text-sm text-slate-800">${result}</pre>`;
        }
    } catch (error) {
        console.error("Code execution error:", error);
        const resultSections = contentRef.value?.querySelectorAll(
            ".code-example-container .bg-white",
        );
        if (resultSections && resultSections.length > 0) {
            const lastResultSection = resultSections[resultSections.length - 1];
            lastResultSection.innerHTML = `<pre class="bg-red-50 p-3 rounded text-sm text-red-800">错误: ${(error as Error).message}</pre>`;
        }
    }
};

const renderedContent = computed(() => {
    const content = props.content.replace(/^---\n[\s\S]*?\n---/, "");
    // 处理代码示例标记
    const processedContent = content.replace(
        /```typescript\s*example\n([\s\S]*?)```/g,
        (_match, code) => {
            return `<pre><code class="language-typescript" data-example="true">${code}</code></pre>`;
        },
    );
    return marked.parse(processedContent) as string;
});
</script>

<style scoped>
.markdown-viewer {
  @apply text-slate-300;
}

.markdown-viewer :deep(h1) {
  @apply text-3xl font-bold mb-6 mt-8 text-white;
}

.markdown-viewer :deep(h2) {
  @apply text-2xl font-semibold mb-4 mt-6 text-white border-b-2 border-blue-500/20 pb-2;
}

.markdown-viewer :deep(h3) {
  @apply text-xl font-semibold mb-3 mt-5 text-white;
}

.markdown-viewer :deep(p) {
  @apply mb-4 leading-relaxed;
}

.markdown-viewer :deep(ul),
.markdown-viewer :deep(ol) {
  @apply mb-4 pl-6;
}

.markdown-viewer :deep(li) {
  @apply mb-2;
}

.markdown-viewer :deep(code:not(pre code)) {
  @apply bg-blue-900/50 px-1.5 py-0.5 rounded text-sm font-mono text-blue-400;
}

.markdown-viewer :deep(pre) {
  @apply p-4 rounded-lg mb-4 overflow-x-auto;
}

.markdown-viewer :deep(pre code) {
  @apply bg-transparent px-0 py-0;
}

.markdown-viewer :deep(a) {
  @apply text-blue-400 hover:text-blue-300 underline;
}

.markdown-viewer :deep(blockquote) {
  @apply border-l-4 border-blue-500 pl-4 italic text-slate-400 mb-4 bg-blue-500/5 py-2;
}

.markdown-viewer :deep(table) {
  @apply w-full border-collapse mb-4;
}

.markdown-viewer :deep(th),
.markdown-viewer :deep(td) {
  @apply border border-blue-800/50 px-4 py-2;
}

.markdown-viewer :deep(th) {
  @apply bg-blue-900/30 font-semibold text-white;
}
</style>

<style>
/* Shiki 代码高亮样式 */
.shiki {
  font-family: 'Fira Code', Consolas, Monaco, 'Andale Mono', 'Ubuntu Mono', monospace;
  font-size: 14px;
  line-height: 1.5;
  border-radius: 0.5rem;
  padding: 1rem;
  overflow-x: auto;
  background: #282c34;
  border: 1px solid rgba(14, 165, 233, 0.2);
}

.shiki code {
  font-family: inherit;
}
</style>