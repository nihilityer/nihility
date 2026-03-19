<template>
  <div class="html-preview">
    <div class="preview-toolbar">
      <el-button
          :icon="Refresh"
          size="small"
          title="刷新预览"
          @click="refreshPreview"
      >
        刷新
      </el-button>
    </div>
    <iframe
        ref="previewFrame"
        :src="previewUrl"
        class="preview-iframe"
        sandbox="allow-scripts allow-same-origin"
        @load="handleLoad"
    />
  </div>
</template>

<script lang="ts" setup>
import {computed, ref, watch} from 'vue'
import {Refresh} from '@element-plus/icons-vue'

const props = defineProps<{
  html: string
  testPath?: string
}>()

const previewFrame = ref<HTMLIFrameElement>()
const refreshKey = ref(Date.now())

// 如果提供了测试路径，使用实际的 /html/{path} 路由进行预览
// 否则使用 data URL 进行本地预览
const previewUrl = computed(() => {
  if (props.testPath) {
    // 使用测试路径访问实际的服务器路由，添加时间戳强制刷新
    return `/html${props.testPath}`
  } else {
    // 使用 data URL 进行本地预览
    return `data:text/html;charset=utf-8,${encodeURIComponent(props.html)}`
  }
})

// 监听 html 和 testPath 变化，自动刷新预览
watch(
    () => [props.html, props.testPath],
    () => {
      refreshPreview()
    }
)

const refreshPreview = () => {
  refreshKey.value = Date.now()
}

const handleLoad = () => {
  // iframe 加载完成
}
</script>

<style scoped>
.html-preview {
  width: 100%;
  height: 500px;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.preview-toolbar {
  padding: 8px;
  border-bottom: 1px solid var(--el-border-color);
  background: var(--el-fill-color-light);
}

.preview-iframe {
  flex: 1;
  width: 100%;
  border: none;
  background: white;
}
</style>
