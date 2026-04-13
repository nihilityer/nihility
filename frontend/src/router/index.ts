import type {RouteRecordRaw} from 'vue-router'
import {createRouter, createWebHashHistory} from 'vue-router'
import {useAuthStore} from '@/stores/auth'
import {login} from '@/api/auth'

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
                path: 'module-configs',
                name: 'ModuleConfigManager',
                component: () => import('@/views/ModuleConfigManager.vue'),
                meta: {title: '模块配置', icon: 'Setting', requiresAuth: true},
            },
            {
                path: 'html-pages',
                name: 'HtmlPageManager',
                component: () => import('@/views/HtmlPageManager.vue'),
                meta: {title: 'HTML 页面管理', icon: 'Document', requiresAuth: true},
            },
            {
                path: 'device-display',
                name: 'DeviceDisplay',
                component: () => import('@/views/device-display/index.vue'),
                meta: {title: '设备展示', icon: 'Monitor', requiresAuth: true},
            },
            {
                path: 'device-display/edge-zectrix',
                name: 'EdgeZectrixDisplay',
                component: () => import('@/views/device-display/EdgeZectrixDisplay.vue'),
                meta: {title: 'ZecTrix 设备', icon: 'Monitor', requiresAuth: true},
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
router.beforeEach(async (to, _from) => {
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

    // 检查 URL 中是否带有 username 和 password 参数
    const usernameParam = to.query.username as string | undefined
    const passwordParam = to.query.password as string | undefined
    if (usernameParam && passwordParam) {
        try {
            const response = await login({username: usernameParam, password: passwordParam})
            authStore.setToken(response.access_token)

            // 从 URL 中移除敏感参数
            const query = {...to.query}
            delete query.username
            delete query.password

            return {
                path: to.path,
                query,
                hash: to.hash,
                replace: true,
            }
        } catch {
            // 登录失败，跳转到登录页
            return '/login'
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
