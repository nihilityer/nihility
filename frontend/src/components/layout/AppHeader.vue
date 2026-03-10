<script setup lang="ts">
import {computed} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import {useLayoutStore} from '@/stores/layout'
import {useAuthStore} from '@/stores/auth'
import {ElMessage} from 'element-plus'

const route = useRoute()
const router = useRouter()
const layoutStore = useLayoutStore()
const authStore = useAuthStore()

const handleToggleSidebar = () => {
  layoutStore.toggleSidebar()
}

const handleLogout = () => {
  authStore.logout()
  ElMessage.success('已退出登录')
  router.push('/login')
}

// 面包屑路径
const breadcrumbs = computed(() => {
  const matched = route.matched.filter((item) => item.meta?.title)
  return matched.map((item) => ({
    title: item.meta.title as string,
    path: item.path,
  }))
})
</script>

<template>
  <div class="app-header-container">
    <div class="header-left">
      <el-button
        class="toggle-button"
        :icon="layoutStore.isCollapsed ? 'Expand' : 'Fold'"
        circle
        @click="handleToggleSidebar"
      />
      <el-breadcrumb separator="/" class="breadcrumb">
        <el-breadcrumb-item v-for="item in breadcrumbs" :key="item.path" :to="item.path">
          {{ item.title }}
        </el-breadcrumb-item>
      </el-breadcrumb>
    </div>

    <div class="header-right">
      <el-dropdown>
        <el-button class="user-button" :icon="'User'" circle />
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item :icon="'SwitchButton'" @click="handleLogout">
              退出登录
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
  </div>
</template>

<style scoped lang="scss">
.app-header-container {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 20px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.toggle-button,
.user-button {
  background-color: rgba(255, 255, 255, 0.2);
  border: none;
  color: #ffffff;

  &:hover {
    background-color: rgba(255, 255, 255, 0.3);
    color: #ffffff;
  }
}

.breadcrumb {
  :deep(.el-breadcrumb__inner) {
    color: rgba(255, 255, 255, 0.8);
    font-weight: normal;

    &:hover {
      color: #ffffff;
    }
  }

  :deep(.el-breadcrumb__separator) {
    color: rgba(255, 255, 255, 0.6);
  }
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}
</style>
