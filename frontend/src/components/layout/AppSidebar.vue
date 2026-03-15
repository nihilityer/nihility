<script lang="ts" setup>
import {computed} from 'vue'
import {useRoute} from 'vue-router'
import {useLayoutStore} from '@/stores/layout'
import {Grid, HomeFilled} from '@element-plus/icons-vue'

const route = useRoute()
const layoutStore = useLayoutStore()

const activeMenu = computed(() => route.path)
</script>

<template>
  <div class="app-sidebar">
    <div class="logo-container">
      <div v-if="!layoutStore.isCollapsed" class="logo-text">Nihility</div>
      <div v-else class="logo-icon">N</div>
    </div>

    <el-menu
        :collapse="layoutStore.isCollapsed"
        :collapse-transition="true"
        :default-active="activeMenu"
        class="sidebar-menu"
        router
    >
      <el-menu-item index="/dashboard">
        <el-icon>
          <HomeFilled/>
        </el-icon>
        <template #title>仪表盘</template>
      </el-menu-item>
      <el-menu-item index="/modules">
        <el-icon>
          <Grid/>
        </el-icon>
        <template #title>模块管理</template>
      </el-menu-item>
    </el-menu>
  </div>
</template>

<style lang="scss" scoped>
.app-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.logo-container {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 60px;
  background-color: #5585a9;
  color: #ffffff;
  font-size: 20px;
  font-weight: 600;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.3s ease;
}

.logo-text {
  letter-spacing: 2px;
}

.logo-icon {
  font-size: 24px;
  font-weight: 700;
}

.sidebar-menu {
  flex: 1;
  border-right: none;
  overflow-y: auto;
  overflow-x: hidden;

  &:not(.el-menu--collapse) {
    width: 200px;
  }
}
</style>
