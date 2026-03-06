<script setup lang="ts">
import {ref} from 'vue'
import {useAuthStore} from '@/stores/auth'

const authStore = useAuthStore()

const stats = ref([
  {
    title: 'API 状态',
    value: '正常',
    icon: 'CircleCheck',
    color: '#67c23a',
  },
  {
    title: 'Token',
    value: '已配置',
    icon: 'Key',
    color: '#5585a9',
  },
  {
    title: '连接状态',
    value: '已连接',
    icon: 'Connection',
    color: '#409eff',
  },
])

const maskedToken = ref(() => {
  const token = authStore.getToken()
  if (!token) return '未配置'
  if (token.length <= 8) return '***'
  return `${token.slice(0, 4)}...${token.slice(-4)}`
})
</script>

<template>
  <div class="dashboard-container">
    <div class="welcome-section">
      <h1 class="welcome-title">欢迎使用 Nihility</h1>
      <p class="welcome-subtitle">为极致上下文利用而生的个人助手</p>
    </div>

    <el-row :gutter="20" class="stats-row">
      <el-col v-for="stat in stats" :key="stat.title" :xs="24" :sm="12" :md="8">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon" :style="{ backgroundColor: stat.color }">
              <el-icon :size="24">
                <component :is="stat.icon" />
              </el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-title">{{ stat.title }}</div>
              <div class="stat-value">{{ stat.value }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20" class="content-row">
      <el-col :span="24">
        <el-card shadow="hover" class="info-card">
          <template #header>
            <div class="card-header">
              <el-icon><InfoFilled /></el-icon>
              <span>系统信息</span>
            </div>
          </template>
          <el-descriptions :column="2" border>
            <el-descriptions-item label="当前 Token">
              <el-text type="info">{{ maskedToken() }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="前端版本">
              <el-tag type="primary">v0.0.0</el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="API 基础路径">
              <el-text>/api</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="主题色">
              <el-tag color="#5585a9" style="color: white">#5585a9</el-tag>
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20" class="content-row">
      <el-col :span="24">
        <el-card shadow="hover" class="quick-start-card">
          <template #header>
            <div class="card-header">
              <el-icon><Promotion /></el-icon>
              <span>快速开始</span>
            </div>
          </template>
          <div class="quick-start-content">
            <el-empty description="暂无快捷操作" :image-size="100" />
            <p class="hint-text">您可以在这里添加常用的 API 操作快捷方式</p>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped lang="scss">
.dashboard-container {
  max-width: 1400px;
  margin: 0 auto;
}

.welcome-section {
  margin-bottom: 32px;
  text-align: center;
}

.welcome-title {
  margin: 0 0 8px;
  font-size: 32px;
  font-weight: 600;
  color: #303133;
}

.welcome-subtitle {
  margin: 0;
  font-size: 16px;
  color: #909399;
}

.stats-row,
.content-row {
  margin-bottom: 20px;
}

.stat-card {
  .stat-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .stat-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 56px;
    height: 56px;
    border-radius: 12px;
    color: white;
  }

  .stat-info {
    flex: 1;
  }

  .stat-title {
    font-size: 14px;
    color: #909399;
    margin-bottom: 4px;
  }

  .stat-value {
    font-size: 20px;
    font-weight: 600;
    color: #303133;
  }
}

.info-card,
.quick-start-card {
  .card-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 16px;
    font-weight: 600;
  }
}

.quick-start-content {
  padding: 40px 20px;
  text-align: center;

  .hint-text {
    margin-top: 16px;
    color: #909399;
    font-size: 14px;
  }
}
</style>
