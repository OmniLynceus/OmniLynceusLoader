<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const pos = ref({ x: 0, y: 0 }); // 初始值可以设为 0

// 定义一个执行随机定位并通知 Rust 的函数
const doRandomize = () => {
    const newX = Math.floor(Math.random() * window.innerWidth);
    const newY = Math.floor(Math.random() * window.innerHeight);
    pos.value = { x: newX, y: newY };
    
    // 考虑 DPI 缩放传给 Rust
    const dpi = window.devicePixelRatio;
    invoke('move_mouse_to', { 
      x: newX * dpi, 
      y: newY * dpi 
    });
};

onMounted(async () => {
  // 1. 注册信号监听（用于后续点击按钮或按 R 键触发）
  await listen('trigger-random', () => {
    doRandomize();
  });

  // 2. 界面加载完成后，立即执行一次，确保圆圈首次出现就与鼠标同步对齐
  doRandomize();
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