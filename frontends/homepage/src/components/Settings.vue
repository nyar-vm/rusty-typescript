<template>
  <div class="settings-container">
    <div class="settings-header">
      <h3 class="settings-title">个性化设置</h3>
      <button @click="close" class="close-button">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    </div>
    
    <div class="settings-content">
      <!-- 主题设置 -->
      <div class="setting-section">
        <h4 class="section-title">主题</h4>
        <div class="theme-options">
          <button 
            v-for="mode in themeModes" 
            :key="mode.value"
            @click="setThemeMode(mode.value)"
            :class="['theme-option', { active: config.mode === mode.value }]"
          >
            <div class="theme-icon">{{ mode.icon }}</div>
            <div class="theme-label">{{ mode.label }}</div>
          </button>
        </div>
      </div>
      
      <!-- 强调色设置 -->
      <div class="setting-section">
        <h4 class="section-title">强调色</h4>
        <div class="accent-colors">
          <button 
            v-for="color in accentColors" 
            :key="color"
            @click="setAccentColor(color)"
            :class="['accent-color', { active: config.accentColor === color }]"
            :style="{ backgroundColor: color }"
          ></button>
        </div>
      </div>
      
      <!-- 字体设置 -->
      <div class="setting-section">
        <h4 class="section-title">字体</h4>
        <div class="font-settings">
          <div class="font-family">
            <label>字体</label>
            <select v-model="selectedFont" @change="setFontFamily(selectedFont)">
              <option v-for="font in fontFamilies" :key="font.value" :value="font.value">
                {{ font.label }}
              </option>
            </select>
          </div>
          <div class="font-size">
            <label>字体大小</label>
            <div class="font-size-control">
              <button @click="decreaseFontSize" class="size-button">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 12H6"></path>
                </svg>
              </button>
              <span class="font-size-value">{{ config.fontSize }}px</span>
              <button @click="increaseFontSize" class="size-button">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 重置设置 -->
      <div class="setting-section">
        <button @click="resetSettings" class="reset-button">
          <span class="reset-icon">↻</span>
          <span>重置所有设置</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { useTheme } from "../composables/useTheme";
import { ThemeMode } from "../types/theme";

defineProps<{
    visible: boolean;
}>();

const emit = defineEmits<{
    close: [];
}>();

const { config, setThemeMode, setAccentColor, setFontSize, setFontFamily } = useTheme();

const themeModes = [
    { value: "light" as ThemeMode, label: "浅色", icon: "☀️" },
    { value: "dark" as ThemeMode, label: "深色", icon: "🌙" },
    { value: "system" as ThemeMode, label: "跟随系统", icon: "⚙️" },
];

const accentColors = [
    "#3b82f6", // 蓝色
    "#8b5cf6", // 紫色
    "#ec4899", // 粉色
    "#10b981", // 绿色
    "#f59e0b", // 橙色
    "#ef4444", // 红色
    "#06b6d4", // 青色
];

const fontFamilies = [
    { value: "Inter, sans-serif", label: "Inter" },
    { value: "Roboto, sans-serif", label: "Roboto" },
    { value: "Open Sans, sans-serif", label: "Open Sans" },
    { value: "Monaco, monospace", label: "Monaco" },
    { value: "Consolas, monospace", label: "Consolas" },
];

const selectedFont = ref(config.value.fontFamily);

watch(
    () => config.value.fontFamily,
    (newFont) => {
        selectedFont.value = newFont;
    },
);

const increaseFontSize = () => {
    if (config.value.fontSize < 20) {
        setFontSize(config.value.fontSize + 1);
    }
};

const decreaseFontSize = () => {
    if (config.value.fontSize > 10) {
        setFontSize(config.value.fontSize - 1);
    }
};

const resetSettings = () => {
    setThemeMode("system");
    setAccentColor("#3b82f6");
    setFontSize(14);
    setFontFamily("Inter, sans-serif");
};

const close = () => {
    emit("close");
};
</script>

<style scoped>
.settings-container {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
  max-width: 400px;
  width: 100%;
  overflow: hidden;
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.settings-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text);
}

.close-button {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-text-secondary);
  padding: 8px;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.close-button:hover {
  background: var(--color-border);
  color: var(--color-text);
}

.settings-content {
  padding: 20px;
}

.setting-section {
  margin-bottom: 24px;
}

.section-title {
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* 主题选项 */
.theme-options {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.theme-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 16px;
  border: 2px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  flex: 1;
  min-width: 80px;
  background: var(--color-surface);
}

.theme-option:hover {
  border-color: var(--color-primary);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.theme-option.active {
  border-color: var(--color-primary);
  background: var(--color-primary);
  color: white;
}

.theme-icon {
  font-size: 24px;
  margin-bottom: 8px;
}

.theme-label {
  font-size: 12px;
  font-weight: 500;
}

/* 强调色选项 */
.accent-colors {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.accent-color {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 3px solid transparent;
}

.accent-color:hover {
  transform: scale(1.1);
}

.accent-color.active {
  border-color: var(--color-text);
  box-shadow: 0 0 0 2px var(--color-surface), 0 0 0 4px var(--color-text);
}

/* 字体设置 */
.font-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.font-family,
.font-size {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.font-family label,
.font-size label {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text);
}

.font-family select {
  padding: 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text);
  font-size: 14px;
  transition: all 0.2s ease;
}

.font-family select:hover {
  border-color: var(--color-primary);
}

.font-size-control {
  display: flex;
  align-items: center;
  gap: 12px;
}

.size-button {
  width: 36px;
  height: 36px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-surface);
  color: var(--color-text);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.size-button:hover {
  border-color: var(--color-primary);
  background: var(--color-primary);
  color: white;
}

.font-size-value {
  font-size: 14px;
  font-weight: 500;
  min-width: 40px;
  text-align: center;
  color: var(--color-text);
}

/* 重置按钮 */
.reset-button {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--color-error);
  border-radius: 6px;
  background: transparent;
  color: var(--color-error);
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  font-weight: 500;
}

.reset-button:hover {
  background: var(--color-error);
  color: white;
}

.reset-icon {
  font-size: 16px;
}
</style>
