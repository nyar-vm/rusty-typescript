<template>
  <div class="editor-skeleton" :class="{ 'dark-mode': isDark }">
    <div class="skeleton-header">
      <div class="skeleton-tab skeleton-shimmer"></div>
      <div class="skeleton-tab skeleton-shimmer"></div>
      <div class="skeleton-tab skeleton-shimmer short"></div>
    </div>
    <div class="skeleton-content">
      <div class="skeleton-line-numbers">
        <div v-for="i in 20" :key="i" class="skeleton-line-number skeleton-shimmer"></div>
      </div>
      <div class="skeleton-code-area">
        <div class="skeleton-line" v-for="(line, index) in skeletonLines" :key="index">
          <div class="skeleton-code-block skeleton-shimmer" :style="{ width: line.width }"></div>
        </div>
      </div>
    </div>
    <div class="skeleton-status-bar">
      <div class="skeleton-status-item skeleton-shimmer"></div>
      <div class="skeleton-status-item skeleton-shimmer short"></div>
      <div class="skeleton-status-item skeleton-shimmer"></div>
    </div>
    <div class="loading-overlay">
      <div class="loading-spinner-container">
        <div class="loading-spinner"></div>
        <div class="loading-text">正在加载编辑器...</div>
        <div class="loading-progress">
          <div class="progress-bar" :style="{ width: progressWidth }"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { useTheme } from "../composables/useTheme";

const { isDark } = useTheme();

const loadingProgress = ref(0);
let progressInterval: number | null = null;

const skeletonLines = [
    { width: "60%" },
    { width: "80%" },
    { width: "45%" },
    { width: "70%" },
    { width: "55%" },
    { width: "85%" },
    { width: "40%" },
    { width: "75%" },
    { width: "50%" },
    { width: "65%" },
    { width: "80%" },
    { width: "35%" },
    { width: "70%" },
    { width: "55%" },
    { width: "90%" },
    { width: "45%" },
    { width: "60%" },
    { width: "75%" },
    { width: "50%" },
    { width: "40%" },
];

const progressWidth = computed(() => `${loadingProgress.value}%`);

onMounted(() => {
    progressInterval = window.setInterval(() => {
        if (loadingProgress.value < 90) {
            loadingProgress.value += Math.random() * 15;
            if (loadingProgress.value > 90) {
                loadingProgress.value = 90;
            }
        }
    }, 300);
});

onUnmounted(() => {
    if (progressInterval) {
        clearInterval(progressInterval);
        progressInterval = null;
    }
});
</script>

<style scoped>
.editor-skeleton {
  width: 100%;
  min-height: 400px;
  background: #1e1e1e;
  border-radius: 8px;
  overflow: hidden;
  position: relative;
  font-family: "Consolas", "Monaco", "Courier New", monospace;
}

.editor-skeleton.dark-mode {
  background: #1e1e1e;
}

.editor-skeleton:not(.dark-mode) {
  background: #ffffff;
}

.skeleton-header {
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  background: #252526;
  border-bottom: 1px solid #3c3c3c;
}

.dark-mode .skeleton-header {
  background: #252526;
  border-bottom-color: #3c3c3c;
}

:not(.dark-mode) .skeleton-header {
  background: #f3f3f3;
  border-bottom-color: #e0e0e0;
}

.skeleton-tab {
  height: 8px;
  width: 80px;
  border-radius: 4px;
  background: #3c3c3c;
}

.skeleton-tab.short {
  width: 50px;
}

.skeleton-content {
  display: flex;
  flex: 1;
  min-height: 320px;
}

.skeleton-line-numbers {
  width: 50px;
  padding: 16px 8px;
  background: #1e1e1e;
  border-right: 1px solid #3c3c3c;
}

.dark-mode .skeleton-line-numbers {
  background: #1e1e1e;
  border-right-color: #3c3c3c;
}

:not(.dark-mode) .skeleton-line-numbers {
  background: #fafafa;
  border-right-color: #e0e0e0;
}

.skeleton-line-number {
  height: 14px;
  width: 24px;
  margin-bottom: 6px;
  border-radius: 2px;
  background: #3c3c3c;
}

.skeleton-code-area {
  flex: 1;
  padding: 16px;
}

.skeleton-line {
  margin-bottom: 6px;
}

.skeleton-code-block {
  height: 14px;
  border-radius: 2px;
  background: #3c3c3c;
}

.dark-mode .skeleton-code-block,
.dark-mode .skeleton-line-number,
.dark-mode .skeleton-tab {
  background: #3c3c3c;
}

:not(.dark-mode) .skeleton-code-block,
:not(.dark-mode) .skeleton-line-number,
:not(.dark-mode) .skeleton-tab {
  background: #e0e0e0;
}

.skeleton-status-bar {
  display: flex;
  gap: 16px;
  padding: 4px 16px;
  background: #007acc;
  align-items: center;
}

.skeleton-status-item {
  height: 8px;
  width: 60px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.3);
}

.skeleton-status-item.short {
  width: 40px;
}

.skeleton-shimmer {
  animation: shimmer 1.5s infinite;
  background: linear-gradient(
    90deg,
    var(--shimmer-color, #3c3c3c) 0%,
    var(--shimmer-highlight, #4a4a4a) 50%,
    var(--shimmer-color, #3c3c3c) 100%
  );
  background-size: 200% 100%;
}

.dark-mode .skeleton-shimmer {
  --shimmer-color: #3c3c3c;
  --shimmer-highlight: #4a4a4a;
}

:not(.dark-mode) .skeleton-shimmer {
  --shimmer-color: #e0e0e0;
  --shimmer-highlight: #f0f0f0;
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(2px);
}

.loading-spinner-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.loading-spinner {
  width: 48px;
  height: 48px;
  border: 4px solid rgba(255, 255, 255, 0.2);
  border-top-color: #007acc;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.loading-text {
  color: #ffffff;
  font-size: 16px;
  font-weight: 500;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

.loading-progress {
  width: 200px;
  height: 4px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: linear-gradient(90deg, #007acc, #00a8ff);
  border-radius: 2px;
  transition: width 0.3s ease;
}
</style>
