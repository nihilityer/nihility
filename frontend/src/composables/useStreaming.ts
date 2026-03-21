import { ref } from 'vue'
import { streamModuleFunction, type StreamOptions } from '@/api/modules'

export function useStreaming() {
    const isStreaming = ref(false)
    const content = ref('')
    const error = ref<string | null>(null)
    let abortFn: (() => void) | null = null

    const startStreaming = (
        moduleType: string,
        funcName: string,
        param: any,
        streamOptions: Omit<StreamOptions, 'onChunk' | 'onDone' | 'onError'> = {}
    ) => {
        isStreaming.value = true
        content.value = ''
        error.value = null

        const { abort } = streamModuleFunction(
            moduleType,
            { func_name: funcName, param, is_mut: false },
            {
                onChunk: (c) => {
                    content.value += c
                },
                onDone: () => {
                    isStreaming.value = false
                },
                onError: (e) => {
                    error.value = e
                    isStreaming.value = false
                },
                ...streamOptions
            }
        )

        abortFn = abort
    }

    const stopStreaming = () => {
        abortFn?.()
        isStreaming.value = false
    }

    return {
        isStreaming,
        content,
        error,
        startStreaming,
        stopStreaming
    }
}
