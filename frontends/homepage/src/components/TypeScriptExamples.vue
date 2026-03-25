<template>
  <div class="typescript-examples">
    <div class="examples-header">
      <h2>TypeScript 示例库</h2>
      <p>浏览和使用 TypeScript 代码示例</p>
    </div>

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
      <div class="sort-controls">
        <select v-model="sortBy" class="sort-select">
          <option value="title">按标题排序</option>
          <option value="category">按分类排序</option>
        </select>
      </div>
    </div>

    <div class="examples-filters">
      <button 
        v-for="category in categories" 
        :key="category.id"
        :class="['category-btn', { active: selectedCategory === category.id }]"
        @click="selectedCategory = category.id"
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

    <div class="examples-grid">
      <div 
        v-for="example in sortedAndFilteredExamples" 
        :key="example.id"
        class="example-card"
      >
        <div class="example-header">
          <h3>{{ example.title }}</h3>
          <div class="example-tags">
            <span 
              v-for="tag in example.tags" 
              :key="tag"
              class="tag"
            >
              {{ tag }}
            </span>
          </div>
        </div>
        <p class="example-description">{{ example.description }}</p>
        <div class="example-actions">
          <button 
            class="import-btn"
            @click="importExample(example)"
          >
            导入到 Playground
          </button>
        </div>
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
const sortBy = ref("title");
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
        if (sortBy.value === "title") {
            return a.title.localeCompare(b.title);
        } else if (sortBy.value === "category") {
            return a.category.localeCompare(b.category);
        }
        return 0;
    });
});

const importExample = (example: any) => {
    emit("import", example.code);
};
</script>

<style scoped>
.typescript-examples {
  width: 100%;
  max-width: 1000px;
  margin: 0 auto;
  padding: 20px;
  background-color: #f5f5f5;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.examples-header {
  text-align: center;
  margin-bottom: 30px;
}

.examples-header h2 {
  color: #333;
  font-size: 28px;
  margin-bottom: 10px;
}

.examples-header p {
  color: #666;
  font-size: 16px;
}

.examples-controls {
  display: flex;
  flex-wrap: wrap;
  gap: 15px;
  justify-content: center;
  align-items: center;
  margin-bottom: 20px;
}

.search-box {
  position: relative;
  max-width: 400px;
}

.sort-controls {
  display: flex;
  align-items: center;
}

.sort-select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  outline: none;
  background-color: white;
  cursor: pointer;
  transition: border-color 0.3s ease;
}

.sort-select:focus {
  border-color: #2196F3;
  box-shadow: 0 0 0 2px rgba(33, 150, 243, 0.2);
}

.search-input {
  width: 100%;
  padding: 12px 16px 12px 40px;
  border: 1px solid #ddd;
  border-radius: 24px;
  font-size: 14px;
  outline: none;
  transition: all 0.3s ease;
}

.search-input:focus {
  border-color: #2196F3;
  box-shadow: 0 0 0 2px rgba(33, 150, 243, 0.2);
}

.search-icon {
  position: absolute;
  left: 14px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: #999;
}

.examples-filters {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: center;
  margin-bottom: 30px;
}

.category-btn {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 20px;
  background-color: white;
  color: #333;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.3s ease;
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

.examples-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.example-card {
  background-color: white;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 20px;
  transition: all 0.3s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.example-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 5px 15px rgba(0, 0, 0, 0.1);
}

.example-header {
  margin-bottom: 10px;
}

.example-header h3 {
  color: #333;
  font-size: 18px;
  margin-bottom: 8px;
}

.example-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag {
  padding: 4px 8px;
  background-color: #f0f0f0;
  border-radius: 12px;
  font-size: 12px;
  color: #666;
}

.example-description {
  color: #666;
  font-size: 14px;
  line-height: 1.4;
  margin-bottom: 15px;
}

.example-actions {
  text-align: right;
}

.import-btn {
  padding: 8px 16px;
  background-color: #4CAF50;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.3s ease;
}

.import-btn:hover {
  background-color: #45a049;
}

.no-examples {
  text-align: center;
  padding: 40px 0;
  color: #666;
  font-size: 16px;
}

@media (max-width: 768px) {
  .examples-grid {
    grid-template-columns: 1fr;
  }
  
  .examples-filters {
    flex-direction: column;
    align-items: center;
  }
  
  .category-btn {
    width: 100%;
    max-width: 200px;
    text-align: center;
  }
}
</style>