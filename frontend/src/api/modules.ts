import http from './http'

export interface ModuleType {
    Embed?: 'BrowserControl' | 'EdgeDeviceControl'
    Wasm?: string
}

export interface FunctionMetadata {
    name: string
    desc: string
    tags: string[]
    params: any
}

export interface ModuleFunctions {
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
