<script setup lang="ts">
import {ref, onMounted} from 'vue'
import {useRouter} from 'vue-router'
import {useAuthStore} from '@/stores/auth'
import {ElMessage} from 'element-plus'

const router = useRouter()
const authStore = useAuthStore()

const loginForm = ref({
  username: '',
  password: '',
  remember: false,
})

const loading = ref(false)

// 加载保存的登录凭据
onMounted(() => {
  const saved = authStore.getSavedCredentials()
  loginForm.value = {
    username: saved.username,
    password: saved.password,
    remember: saved.remember,
  }
})

const handleLogin = async () => {
  if (!loginForm.value.username.trim()) {
    ElMessage.warning('请输入用户名')
    return
  }

  if (!loginForm.value.password) {
    ElMessage.warning('请输入密码')
    return
  }

  loading.value = true

  try {
    await authStore.login({
      username: loginForm.value.username.trim(),
      password: loginForm.value.password,
      remember: loginForm.value.remember,
    })
    ElMessage.success('登录成功')
    router.push('/dashboard')
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '登录失败，请检查用户名和密码')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="login-container">
    <el-card class="login-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <el-icon class="header-icon" :size="32"><User /></el-icon>
          <h2 class="card-title">用户登录</h2>
        </div>
      </template>

      <div class="card-content">
        <p class="description">
          请使用您的账号和密码登录系统
        </p>

        <el-form :model="loginForm" class="login-form">
          <el-form-item label="用户名">
            <el-input
              v-model="loginForm.username"
              placeholder="请输入用户名"
              clearable
              size="large"
              @keyup.enter="handleLogin"
            >
              <template #prefix>
                <el-icon><User /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item label="密码">
            <el-input
              v-model="loginForm.password"
              type="password"
              placeholder="请输入密码"
              show-password
              clearable
              size="large"
              @keyup.enter="handleLogin"
            >
              <template #prefix>
                <el-icon><Lock /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item>
            <el-checkbox v-model="loginForm.remember">
              记住密码
            </el-checkbox>
          </el-form-item>

          <el-form-item>
            <el-button
              type="primary"
              size="large"
              :loading="loading"
              class="login-button"
              @click="handleLogin"
            >
              登录
            </el-button>
          </el-form-item>
        </el-form>
      </div>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.login-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: calc(100vh - 160px);
  padding: 20px;
}

.login-card {
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

.login-form {
  margin-top: 32px;
}

.login-button {
  width: 100%;
}
</style>
