import {defineStore} from 'pinia'
import {ref} from 'vue'
import {login as loginApi, type LoginRequest} from '@/api/auth'

const TOKEN_KEY = 'nihility_auth_token'
const USERNAME_KEY = 'nihility_username'
const PASSWORD_KEY = 'nihility_password'
const REMEMBER_KEY = 'nihility_remember_password'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string>('')
  const username = ref<string>('')
  const isAuthenticated = ref(false)

  // 从 localStorage 初始化
  const init = () => {
    const savedToken = localStorage.getItem(TOKEN_KEY)
    const savedUsername = localStorage.getItem(USERNAME_KEY)

    if (savedToken) {
      token.value = savedToken
      isAuthenticated.value = true
    }
    if (savedUsername) {
      username.value = savedUsername
    }
  }

  // 获取 token
  const getToken = (): string => {
    if (!token.value) {
      init()
    }
    return token.value
  }

  // 设置 token
  const setToken = (newToken: string) => {
    token.value = newToken
    isAuthenticated.value = true
    localStorage.setItem(TOKEN_KEY, newToken)
  }

  // 清除 token
  const clearToken = () => {
    token.value = ''
    username.value = ''
    isAuthenticated.value = false
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(USERNAME_KEY)
  }

  // 检查是否已配置 token
  const hasToken = (): boolean => {
    return !!getToken()
  }

  // 获取保存的登录凭据
  const getSavedCredentials = (): { username: string; password: string; remember: boolean } => {
    const remember = localStorage.getItem(REMEMBER_KEY) === 'true'
    return {
      username: localStorage.getItem(USERNAME_KEY) || '',
      password: remember ? localStorage.getItem(PASSWORD_KEY) || '' : '',
      remember,
    }
  }

  // 保存登录凭据（如果记住密码）
  const saveCredentials = (user: string, password: string, remember: boolean) => {
    localStorage.setItem(USERNAME_KEY, user)
    localStorage.setItem(REMEMBER_KEY, String(remember))

    if (remember) {
      localStorage.setItem(PASSWORD_KEY, password)
    } else {
      localStorage.removeItem(PASSWORD_KEY)
    }
  }

  // 登录
  const login = async (credentials: LoginRequest & { remember?: boolean }) => {
    const response = await loginApi({
      username: credentials.username,
      password: credentials.password,
    })

    // 保存 token 和用户信息
    setToken(response.access_token)
    username.value = credentials.username

    // 保存登录凭据（如果选择记住密码）
    saveCredentials(
      credentials.username,
      credentials.password,
      credentials.remember ?? false
    )

    return response
  }

  // 登出
  const logout = () => {
    clearToken()
  }

  // 初始化
  init()

  return {
    token,
    username,
    isAuthenticated,
    getToken,
    setToken,
    clearToken,
    hasToken,
    getSavedCredentials,
    login,
    logout,
  }
})
