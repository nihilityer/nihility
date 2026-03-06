import {defineStore} from 'pinia'
import {computed, ref} from 'vue'

export const useLayoutStore = defineStore('layout', () => {
  const isCollapsed = ref(false)
  const isMobile = ref(false)

  // 计算侧边栏宽度
  const sidebarWidth = computed(() => {
    return isCollapsed.value ? '64px' : '200px'
  })

  // 切换侧边栏
  const toggleSidebar = () => {
    isCollapsed.value = !isCollapsed.value
  }

  // 设置侧边栏折叠状态
  const setCollapsed = (value: boolean) => {
    isCollapsed.value = value
  }

  // 设置移动端状态
  const setMobile = (value: boolean) => {
    isMobile.value = value
  }

  // 响应式断点检测
  const handleResize = () => {
    const mobile = window.innerWidth < 768
    setMobile(mobile)
    // 移动端自动折叠侧边栏
    if (mobile && !isCollapsed.value) {
      setCollapsed(true)
    }
  }

  // 初始化响应式检测
  const initResponsive = () => {
    handleResize()
    window.addEventListener('resize', handleResize)
  }

  return {
    isCollapsed,
    isMobile,
    sidebarWidth,
    toggleSidebar,
    setCollapsed,
    setMobile,
    initResponsive,
  }
})
