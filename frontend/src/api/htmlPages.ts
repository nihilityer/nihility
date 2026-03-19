import http from './http'

export interface HtmlPage {
    id: string
    path: string
    html: string
    update_at: string
}

export interface HtmlPageSummary {
    id: string
    path: string
    update_at: string
}

export interface HtmlPageListResponse {
    pages: HtmlPageSummary[]
    total: number
}

export interface HtmlPageRequest {
    path: string
    html: string
}

export const listHtmlPages = () => {
    return http.get<HtmlPageListResponse>('/html-pages')
}

export const getHtmlPage = (id: string) => {
    return http.get<HtmlPage>(`/html-pages/${id}`)
}

export const createHtmlPage = (data: HtmlPageRequest) => {
    return http.post<HtmlPage>('/html-pages', data)
}

export const updateHtmlPage = (id: string, data: HtmlPageRequest) => {
    return http.put<HtmlPage>(`/html-pages/${id}`, data)
}

export const deleteHtmlPage = (id: string) => {
    return http.delete(`/html-pages/${id}`)
}
