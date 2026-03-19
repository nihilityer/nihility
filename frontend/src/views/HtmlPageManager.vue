<template>
  <div class="html-page-manager">
    <el-card class="header-card">
      <template #header>
        <div class="card-header">
          <span>HTML 页面管理</span>
          <el-button type="primary" @click="handleCreate">
            <el-icon>
              <Plus/>
            </el-icon>
            新建页面
          </el-button>
        </div>
      </template>
    </el-card>

    <el-row :gutter="20" style="margin-top: 20px">
      <!-- 左侧：页面列表 -->
      <el-col :span="8">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>页面列表</span>
              <el-button :loading="loading" size="small" type="primary" @click="loadPageList">
                刷新
              </el-button>
            </div>
          </template>
          <el-table
              v-loading="loading"
              :data="pageList"
              highlight-current-row
              style="width: 100%"
              @current-change="handleSelectPage"
          >
            <el-table-column label="路径" min-width="120" prop="path"/>
            <el-table-column label="更新时间" min-width="100" prop="update_at">
              <template #default="{row}">
                {{ formatDate(row.update_at) }}
              </template>
            </el-table-column>
            <el-table-column align="center" label="操作" width="80">
              <template #default="{row}">
                <el-button size="small" type="danger" @click.stop="handleDelete(row)">
                  删除
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>

      <!-- 右侧：编辑区 -->
      <el-col :span="16">
        <el-card v-if="selectedPage">
          <el-tabs v-model="activeTab">
            <!-- 编辑器标签页 -->
            <el-tab-pane label="编辑器" name="editor">
              <el-form label-width="80px">
                <el-form-item label="路径">
                  <el-input v-model="formData.path" placeholder="/example"/>
                </el-form-item>
                <el-form-item label="HTML">
                  <HtmlEditor v-model="formData.html"/>
                </el-form-item>
              </el-form>
            </el-tab-pane>

            <!-- 预览标签页 -->
            <el-tab-pane label="预览" name="preview">
              <el-form label-width="100px" style="margin-bottom: 16px">
                <el-form-item label="测试路径">
                  <el-input
                      v-model="testPath"
                      placeholder="/test?username=admin&password=123456"
                  >
                    <template #append>
                      <el-button @click="handleTestInNewTab">新标签页打开</el-button>
                    </template>
                  </el-input>
                  <template #extra>
                    <span style="font-size: 12px; color: var(--el-text-color-secondary)">
                      输入完整的测试路径（包括查询参数），预览将使用此路径访问服务器
                    </span>
                  </template>
                </el-form-item>
              </el-form>
              <HtmlPreview :html="formData.html" :test-path="testPath"/>
            </el-tab-pane>
          </el-tabs>

          <!-- 操作按钮 -->
          <div class="actions" style="margin-top: 20px">
            <el-button :loading="saving" type="primary" @click="handleSave"> 保存</el-button>
            <el-button @click="handleCancel">取消</el-button>
          </div>
        </el-card>

        <!-- 空状态 -->
        <el-card v-else>
          <el-empty description="请选择或创建一个页面"/>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script lang="ts" setup>
import {computed, onMounted, ref} from 'vue'
import {ElMessage, ElMessageBox} from 'element-plus'
import {Plus} from '@element-plus/icons-vue'
import HtmlEditor from '@/components/HtmlEditor.vue'
import HtmlPreview from '@/components/HtmlPreview.vue'
import {
  createHtmlPage,
  deleteHtmlPage,
  getHtmlPage,
  type HtmlPage,
  type HtmlPageSummary,
  listHtmlPages,
  updateHtmlPage,
} from '@/api/htmlPages'

const loading = ref(false)
const saving = ref(false)
const pageList = ref<HtmlPageSummary[]>([])
const selectedPage = ref<HtmlPageSummary | null>(null)
const selectedPageDetail = ref<HtmlPage | null>(null)
const activeTab = ref('editor')
const testPath = ref('')

const formData = ref({
  path: '',
  html: '',
})

const isCreateMode = ref(false)

// 格式化日期
const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN')
}

// 计算完整测试 URL
const fullTestUrl = computed(() => {
  if (!testPath.value) {
    return ''
  }
  const baseUrl = window.location.origin
  return `${baseUrl}/html${testPath.value}`
})

// 加载页面列表
const loadPageList = async () => {
  loading.value = true
  try {
    const response = await listHtmlPages()
    pageList.value = response.data.pages
  } catch (error) {
    ElMessage.error('加载页面列表失败')
  } finally {
    loading.value = false
  }
}

// 选择页面
const handleSelectPage = async (page: HtmlPageSummary | null) => {
  if (!page) return

  selectedPage.value = page
  isCreateMode.value = false

  try {
    const response = await getHtmlPage(page.id)
    selectedPageDetail.value = response.data
    formData.value = {
      path: response.data.path,
      html: response.data.html,
    }
    testPath.value = '/' + response.data.path
  } catch (error) {
    ElMessage.error('加载页面详情失败')
  }
}

// 创建新页面
const handleCreate = () => {
  selectedPage.value = {id: '', path: '', update_at: ''} // 占位对象
  selectedPageDetail.value = null
  isCreateMode.value = true
  formData.value = {
    path: '',
    html: '<!DOCTYPE html>\n<html>\n<head>\n  <title>New Page</title>\n</head>\n<body>\n  <h1>Hello World</h1>\n</body>\n</html>',
  }
  testPath.value = ''
  activeTab.value = 'editor'
}

// 表单验证
const validateForm = (): boolean => {
  if (!formData.value.path) {
    ElMessage.error('路径不能为空')
    return false
  }

  if (!formData.value.html.trim()) {
    ElMessage.error('HTML 内容不能为空')
    return false
  }

  return true
}

// 保存页面
const handleSave = async () => {
  if (!validateForm()) return

  saving.value = true
  try {
    if (isCreateMode.value) {
      // 创建新页面
      const response = await createHtmlPage({
        path: formData.value.path,
        html: formData.value.html,
      })
      ElMessage.success('创建成功')
      // 刷新列表并选中新创建的页面
      await loadPageList()
      const newPage = pageList.value.find((p) => p.id === response.data.id)
      if (newPage) {
        await handleSelectPage(newPage)
      }
    } else if (selectedPageDetail.value) {
      // 更新现有页面
      await updateHtmlPage(selectedPageDetail.value.id, {
        path: formData.value.path,
        html: formData.value.html,
      })
      ElMessage.success('保存成功')
      // 刷新列表
      await loadPageList()
      // 保持当前选中状态
      if (selectedPage.value) {
        const updatedPage = pageList.value.find((p) => p.id === selectedPage.value?.id)
        if (updatedPage) {
          selectedPage.value = updatedPage
        }
      }
    }
  } catch (error: any) {
    const errorMsg = error.response?.data || '保存失败'
    ElMessage.error(errorMsg)
  } finally {
    saving.value = false
  }
}

// 删除页面
const handleDelete = async (page: HtmlPageSummary) => {
  try {
    await ElMessageBox.confirm(`确定要删除页面 "${page.path}" 吗？`, '删除确认', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })

    await deleteHtmlPage(page.id)
    ElMessage.success('删除成功')

    // 如果删除的是当前选中的页面，清空选中状态
    if (selectedPage.value?.id === page.id) {
      selectedPage.value = null
      selectedPageDetail.value = null
    }

    // 刷新列表
    await loadPageList()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
    }
  }
}

// 取消编辑
const handleCancel = () => {
  selectedPage.value = null
  selectedPageDetail.value = null
  formData.value = {path: '', html: ''}
}

// 在新标签页中测试
const handleTestInNewTab = () => {
  if (!fullTestUrl.value) {
    ElMessage.warning('请先输入测试路径')
    return
  }
  window.open(fullTestUrl.value, '_blank')
}

// 复制到剪贴板
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    ElMessage.success('已复制到剪贴板')
  } catch (error) {
    ElMessage.error('复制失败')
  }
}

onMounted(() => {
  loadPageList()
})
</script>

<style scoped>
.html-page-manager {
  padding: 20px;
}

.header-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.actions {
  display: flex;
  gap: 10px;
}
</style>
