import http from './http'
import {useAuthStore} from '@/stores/auth'

// ModuleType is now a string in format: "embed-{module}" or "wasm-{path}"
export type ModuleType = string

// JSON Schema 类型定义
export interface JSONSchema {
    $schema?: string
    title?: string
    type?: string | string[]
    properties?: Record<string, any>
    required?: string[]
    default?: any
    [key: string]: any
}

export interface FunctionMetadata {
    name: string
    desc: string
    tags: string[]
    params: JSONSchema
}

export interface ModuleFunctions {
    description: string
    no_perm_func: FunctionMetadata[]
    perm_func: FunctionMetadata[]
}

export interface CallRequest {
    func_name: string
    param: any
    is_mut: boolean
}

export interface CallResponse {
    result: any
}

// SSE Streaming types
export interface StreamChunk {
    content: string
    error?: string
}

export type StreamEventType = 'chunk' | 'done' | 'error'

export interface StreamOptions {
    onChunk?: (content: string) => void
    onDone?: () => void
    onError?: (error: string) => void
}

// 获取已加载的模块列表
export const getLoadedModules = () => {
    return http.get<ModuleType[]>('/modules')
}

// 查询所有模块的功能列表
export const queryAllFunctions = () => {
    return http.get<Record<string, ModuleFunctions>>('/modules/functions')
}

// 查询指定模块的功能列表
export const queryModuleFunctions = (moduleType: string) => {
    return http.get<ModuleFunctions>(`/modules/${moduleType}/functions`)
}

// 调用指定模块的方法
export const callModuleFunction = (moduleType: string, request: CallRequest) => {
    return http.post<CallResponse>(`/modules/${moduleType}/call`, request)
}

// 流式调用指定模块的方法 (SSE)
export const streamModuleFunction = (
    moduleType: string,
    request: CallRequest,
    options: StreamOptions = {}
): { abort: () => void } => {
    const controller = new AbortController()
    const authStore = useAuthStore()
    const token = authStore.getToken()

    const headers: Record<string, string> = {
        'Content-Type': 'application/json',
    }
    if (token) {
        headers['Authorization'] = `Bearer ${token}`
    }

    fetch(`/api/modules/${moduleType}/stream`, {
        method: 'POST',
        headers,
        body: JSON.stringify(request),
        signal: controller.signal
    })
        .then(async response => {
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`)
            }

            const reader = response.body?.getReader()
            const decoder = new TextDecoder()

            if (!reader) {
                options.onError?.('No reader available')
                return
            }

            const read = () => {
                reader.read().then(({ done, value }) => {
                    if (done) {
                        options.onDone?.()
                        return
                    }

                    const chunk = decoder.decode(value)
                    // Parse SSE events
                    const lines = chunk.split('\n')
                    for (const line of lines) {
                        if (line.startsWith('data: ')) {
                            try {
                                const data = JSON.parse(line.slice(6))
                                if (data.error) {
                                    options.onError?.(data.error)
                                } else if (data.content === '') {
                                    options.onDone?.()
                                } else {
                                    options.onChunk?.(data.content)
                                }
                            } catch (e) {
                                // Ignore parse errors for incomplete JSON
                            }
                        }
                    }
                    read()
                }).catch(err => {
                    if (err.name !== 'AbortError') {
                        options.onError?.(err.message)
                    }
                })
            }

            read()
        })
        .catch(err => {
            if (err.name !== 'AbortError') {
                options.onError?.(err.message)
            }
        })

    return {
        abort: () => controller.abort()
    }
}
