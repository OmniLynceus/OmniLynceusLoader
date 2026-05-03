<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';

const pos = ref({ x: 0, y: 0 }); // 初始值可以设为 0

const transferPixel = (x: number, y: number) => {
  const dpr = window.devicePixelRatio;
  return {
    x: Math.round(x / dpr),
    y: Math.round(y / dpr)
  };
};

onMounted(async () => {
  await listen('move-to', (event) => {
    const { x, y } = event?.payload as { x: number, y: number };
    pos.value = transferPixel(x, y);
  });
});
</script>

<template>
  <div 
    class="red-circle" 
    :style="{ 
      left: pos.x + 'px', 
      top: pos.y + 'px',
      position: 'absolute' 
    }"
  ></div>
</template>

<style scoped>
.red-circle {
  width: 10px;
  height: 10px;
  border: 2px solid #ff0000;
  border-radius: 50%;
  pointer-events: none; /* 确保不收集鼠标事件 */
  transform: translate(-50%, -50%); /* 让坐标点处于圆心 */
}
</style>