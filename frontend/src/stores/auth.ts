import {defineStore} from 'pinia'
import {ref} from 'vue'

const TOKEN_KEY = 'nihility_auth_token'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string>('')

  // 从 localStorage 初始化 token
  const initToken = () => {
    const savedToken = localStorage.getItem(TOKEN_KEY)
    if (savedToken) {
      token.value = savedToken
    }
  }

  // 获取 token
  const getToken = (): string => {
    if (!token.value) {
      initToken()
    }
    return token.value
  }

  // 设置 token
  const setToken = (newToken: string) => {
    token.value = newToken
    localStorage.setItem(TOKEN_KEY, newToken)
  }

  // 清除 token
  const clearToken = () => {
    token.value = ''
    localStorage.removeItem(TOKEN_KEY)
  }

  // 检查是否已配置 token
  const hasToken = (): boolean => {
    return !!getToken()
  }

  // 初始化
  initToken()

  return {
    token,
    getToken,
    setToken,
    clearToken,
    hasToken,
  }
})
