import http from './http'

export interface ModuleConfigSummary {
    id: string
    module_name: string
    updated_at: string
}

export interface ModuleConfig {
    id: string
    module_name: string
    config_value: Record<string, any>
    json_schema: Record<string, any>
    created_at: string
    updated_at: string
}

export interface ModuleConfigListResponse {
    configs: ModuleConfigSummary[]
    total: number
}

export interface ModuleConfigUpdateRequest {
    config_value: Record<string, any>
}

export interface ModuleConfigCreateRequest {
    module_name: string
    config_value: Record<string, any>
    json_schema: Record<string, any>
}

export const listModuleConfigs = () => {
    return http.get<ModuleConfigListResponse>('/module-configs')
}

export const getModuleConfig = (moduleName: string) => {
    return http.get<ModuleConfig>(`/module-configs/${moduleName}`)
}

export const updateModuleConfig = (id: string, data: ModuleConfigUpdateRequest) => {
    return http.put<ModuleConfig>(`/module-configs/id/${id}`, data)
}

export const createModuleConfig = (data: ModuleConfigCreateRequest) => {
    return http.post<ModuleConfig>('/module-configs', data)
}
