<script setup lang="ts">
import {onMounted} from 'vue'
import {useLayoutStore} from '@/stores/layout'
import AppHeader from '@/components/layout/AppHeader.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'

const layoutStore = useLayoutStore()

onMounted(() => {
  // 初始化响应式检测
  layoutStore.initResponsive()
})
</script>

<template>
  <el-container class="app-layout">
    <el-aside :width="layoutStore.sidebarWidth" class="app-aside">
      <AppSidebar />
    </el-aside>
    <el-container class="app-main-container">
      <el-header height="60px" class="app-header">
        <AppHeader />
      </el-header>
      <el-main class="app-main">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<style scoped lang="scss">
.app-layout {
  height: 100%;
  width: 100%;
}

.app-aside {
  background-color: #ffffff;
  border-right: 1px solid #e4e7ed;
  transition: width 0.3s ease;
  overflow: hidden;
}

.app-main-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.app-header {
  background-color: #5585a9;
  padding: 0;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.08);
  z-index: 100;
}

.app-main {
  background-color: #f5f7fa;
  padding: 20px;
  overflow-y: auto;
}
</style>
