<template>
  <div class="typescript-examples">
    <div class="examples-controls">
      <div class="search-box">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索示例..."
          class="search-input"
        />
        <svg class="search-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path>
        </svg>
      </div>
    </div>

    <div class="examples-filters">
      <button
        v-for="category in categories"
        :key="category.id"
        :class="['category-btn', { active: selectedCategory === category.id }]"
        @click="selectedCategory = category.id"
        :title="category.description"
      >
        {{ category.name }}
      </button>
      <button
        :class="['category-btn', { active: selectedCategory === 'all' }]"
        @click="selectedCategory = 'all'"
      >
        全部
      </button>
    </div>

    <div class="examples-list">
      <div
        v-for="example in sortedAndFilteredExamples"
        :key="example.id"
        class="example-card"
      >
        <div class="example-header">
          <h4 class="example-title">{{ example.title }}</h4>
          <div class="example-tags">
            <span
              v-for="tag in example.tags.slice(0, 2)"
              :key="tag"
              class="tag"
            >
              {{ tag }}
            </span>
          </div>
        </div>
        <p class="example-description">{{ example.description }}</p>
        <button
          class="import-btn"
          @click="importExample(example)"
        >
          导入到 Playground
        </button>
      </div>
    </div>

    <div v-if="sortedAndFilteredExamples.length === 0" class="no-examples">
      <p>没有找到匹配的示例</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import examplesData from "../data/examples.json";

const emit = defineEmits<{
    (e: "import", code: string): void;
}>();

const searchQuery = ref("");
const selectedCategory = ref("all");
const categories = examplesData.categories;
const examples = examplesData.examples;

const filteredExamples = computed(() => {
    return examples.filter((example) => {
        const matchesSearch =
            example.title.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
            example.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
            example.tags.some((tag) => tag.toLowerCase().includes(searchQuery.value.toLowerCase()));

        const matchesCategory =
            selectedCategory.value === "all" || example.category === selectedCategory.value;

        return matchesSearch && matchesCategory;
    });
});

const sortedAndFilteredExamples = computed(() => {
    const filtered = filteredExamples.value;

    return [...filtered].sort((a, b) => {
        return a.title.localeCompare(b.title);
    });
});

const importExample = (example: { code: string }) => {
    emit("import", example.code);
};
</script>

<style scoped>
.typescript-examples {
  width: 100%;
}

.examples-controls {
  margin-bottom: 12px;
}

.search-box {
  position: relative;
  width: 100%;
}

.search-input {
  width: 100%;
  padding: 10px 12px 10px 36px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  font-size: 13px;
  outline: none;
  transition: all 0.2s ease;
  background: #fafafa;
}

.search-input:focus {
  border-color: #2196F3;
  box-shadow: 0 0 0 2px rgba(33, 150, 243, 0.1);
  background: white;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 14px;
  height: 14px;
  color: #999;
}

.examples-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}

.category-btn {
  padding: 6px 12px;
  border: 1px solid #e0e0e0;
  border-radius: 16px;
  background-color: white;
  color: #666;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.category-btn:hover {
  border-color: #2196F3;
  color: #2196F3;
}

.category-btn.active {
  background-color: #2196F3;
  border-color: #2196F3;
  color: white;
}

.examples-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.example-card {
  background-color: white;
  border: 1px solid #e8e8e8;
  border-radius: 8px;
  padding: 12px;
  transition: all 0.2s ease;
}

.example-card:hover {
  border-color: #2196F3;
  box-shadow: 0 2px 8px rgba(33, 150, 243, 0.1);
}

.example-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 6px;
  gap: 8px;
}

.example-title {
  color: #333;
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  flex: 1;
}

.example-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  flex-shrink: 0;
}

.tag {
  padding: 2px 6px;
  background-color: #f0f4f8;
  border-radius: 8px;
  font-size: 10px;
  color: #666;
}

.example-description {
  color: #666;
  font-size: 12px;
  line-height: 1.4;
  margin: 0 0 10px 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.import-btn {
  width: 100%;
  padding: 8px 12px;
  background-color: #4CAF50;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s ease;
}

.import-btn:hover {
  background-color: #45a049;
}

.no-examples {
  text-align: center;
  padding: 24px 0;
  color: #999;
  font-size: 14px;
}
</style>
