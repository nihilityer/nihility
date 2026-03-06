<script setup lang="ts">
import {ref} from 'vue'
import {useRouter} from 'vue-router'
import {useAuthStore} from '@/stores/auth'
import {ElMessage} from 'element-plus'

const router = useRouter()
const authStore = useAuthStore()

const tokenForm = ref({
  token: '',
})

const loading = ref(false)

const handleSave = () => {
  if (!tokenForm.value.token.trim()) {
    ElMessage.warning('请输入 Token')
    return
  }

  loading.value = true

  setTimeout(() => {
    authStore.setToken(tokenForm.value.token.trim())
    ElMessage.success('Token 配置成功')
    loading.value = false
    router.push('/dashboard')
  }, 300)
}
</script>

<template>
  <div class="token-config-container">
    <el-card class="token-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <el-icon class="header-icon" :size="32"><Key /></el-icon>
          <h2 class="card-title">Token 配置</h2>
        </div>
      </template>

      <div class="card-content">
        <p class="description">
          请输入您的 API Token 以访问后端服务。Token 将安全存储在本地，所有 API
          请求都会自动携带此 Token。
        </p>

        <el-form :model="tokenForm" class="token-form">
          <el-form-item label="API Token">
            <el-input
              v-model="tokenForm.token"
              type="password"
              placeholder="请输入您的 API Token"
              show-password
              clearable
              size="large"
              @keyup.enter="handleSave"
            />
          </el-form-item>

          <el-form-item>
            <el-button
              type="primary"
              size="large"
              :loading="loading"
              class="save-button"
              @click="handleSave"
            >
              保存并继续
            </el-button>
          </el-form-item>
        </el-form>
      </div>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.token-config-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: calc(100vh - 160px);
  padding: 20px;
}

.token-card {
  width: 100%;
  max-width: 500px;
}

.card-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.header-icon {
  color: #5585a9;
}

.card-title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.card-content {
  padding: 20px 0;
}

.description {
  margin: 0 0 24px;
  color: #606266;
  line-height: 1.6;
  text-align: center;
}

.token-form {
  margin-top: 32px;
}

.save-button {
  width: 100%;
}
</style>
