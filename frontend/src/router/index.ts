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
                meta: {title: '仪表盘', icon: 'HomeFilled', requiresAuth: true},
            },
            {
                path: 'modules',
                name: 'ModuleManager',
                component: () => import('@/views/ModuleManager.vue'),
                meta: {title: '模块管理', icon: 'Grid', requiresAuth: true},
            },
            {
                path: 'login',
                name: 'Login',
                component: () => import('@/views/Login.vue'),
                meta: {title: '登录', icon: 'User', requiresAuth: false},
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

    // 检查 URL 中是否带有 token 参数
    const tokenParam = to.query.token as string | undefined
    if (tokenParam) {
        // 保存 token 到本地缓存
        authStore.setToken(tokenParam)

        // 从 URL 中移除 token 参数（安全考虑）
        const query = {...to.query}
        delete query.token

        // 重定向到相同路径但移除 token 参数
        return {
            path: to.path,
            query,
            hash: to.hash,
            replace: true, // 替换历史记录，避免后退时重新处理 token
        }
    }

    const hasToken = authStore.hasToken()

    // 只检查需要认证的页面，未配置 token 时跳转到登录页
    if (to.meta.requiresAuth && !hasToken) {
        return '/login'
    }
    // 允许已登录用户访问登录页
})

export default router
