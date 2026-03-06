import type {RouteRecordRaw} from 'vue-router'
import {createRouter, createWebHashHistory} from 'vue-router'
import {useAuthStore} from '@/stores/auth'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('@/components/layout/AppLayout.vue'),
    redirect: '/dashboard',
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/Dashboard.vue'),
        meta: { title: '仪表盘', icon: 'HomeFilled', requiresAuth: true },
      },
      {
        path: 'token-config',
        name: 'TokenConfig',
        component: () => import('@/views/TokenConfig.vue'),
        meta: { title: 'Token 配置', icon: 'Key', requiresAuth: false },
      },
    ],
  },
]

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes,
})

// 路由守卫 - 检查 token
router.beforeEach((to, _from) => {
  const authStore = useAuthStore()
  const hasToken = authStore.hasToken()

  // 只检查需要认证的页面，未配置 token 时跳转到配置页
  if (to.meta.requiresAuth && !hasToken) {
    return '/token-config'
  }
  // 允许已登录用户访问 token 配置页以修改 token
})

export default router
