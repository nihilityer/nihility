import {fileURLToPath, URL} from 'node:url'

import {defineConfig} from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
    plugins: [
        vue(),
        vueDevTools(),
    ],
    resolve: {
        alias: {
            '@': fileURLToPath(new URL('./src', import.meta.url))
        },
    },
    server: {
        port: 5173,
        proxy: {
            // 代理 /api 请求到后端服务器
            '/api': {
                target: 'http://localhost:8080',
                changeOrigin: true,
            },
            // 代理 /auth 请求到后端服务器
            '/auth': {
                target: 'http://localhost:8080',
                changeOrigin: true,
            },
            // 代理 /html 请求到后端服务器
            '/html': {
                target: 'http://localhost:8080',
                changeOrigin: true,
            },
        },
    },
})
