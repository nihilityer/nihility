import http from './http'
import axios from "axios";

export interface LoginRequest {
    username: string
    password: string
}

export interface LoginResponse {
    access_token: string
    token_type: string
}

/**
 * 用户登录
 */
export const login = async (data: LoginRequest): Promise<LoginResponse> => {
    const response = await axios.create({
        baseURL: '/',
        timeout: 10000,
        headers: {
            'Content-Type': 'application/json',
        },
    }).post<LoginResponse>('/auth', data)
    return response.data
}

/**
 * 验证 token 是否有效
 */
export const verifyToken = async (): Promise<boolean> => {
    try {
        await http.get('/api/test')
        return true
    } catch {
        return false
    }
}
