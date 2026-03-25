<template>
  <div>
    <div 
      :class="[
        'flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer transition-all',
        'text-slate-700 hover:bg-blue-100 hover:text-blue-800'
      ]"
      @click="node.children ? toggle() : handleSelect()"
    >
      <span v-if="node.children" class="text-sm text-blue-600">
        {{ isExpanded ? '▼' : '▶' }}
      </span>
      <span v-else class="text-sm w-4"></span>
      <span class="flex-1">{{ node.title }}</span>
    </div>
    <div v-if="node.children && isExpanded" class="ml-4 mt-1">
      <DocTreeNode 
        v-for="child in node.children" 
        :key="child.id"
        :node="child"
        @select="(id: string, title: string) => $emit('select', id, title)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

interface DocNode {
    id: string;
    title: string;
    children?: DocNode[];
}

interface Props {
    node: DocNode;
}

const props = defineProps<Props>();
const emit = defineEmits<{
    select: [id: string, title: string];
}>();

const isExpanded = ref(true);

function toggle() {
    isExpanded.value = !isExpanded.value;
}

function handleSelect() {
    emit("select", props.node.id, props.node.title);
}
</script>