<template>
  <div class="module-config-manager">
    <el-card class="header-card">
      <h2>模块配置管理</h2>
      <p>查看和编辑数据库中存储的模块配置</p>
    </el-card>

    <el-row :gutter="20" style="margin-top: 20px">
      <!-- 左侧：模块列表 -->
      <el-col :span="8">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>模块配置列表</span>
              <el-button :loading="loading" size="small" type="primary" @click="loadConfigs">
                刷新
              </el-button>
            </div>
          </template>
          <el-menu :default-active="selectedModuleName" @select="handleModuleSelect">
            <el-menu-item
                v-for="config in configs"
                :key="config.module_name"
                :index="config.module_name"
            >
              <el-icon>
                <Setting/>
              </el-icon>
              <span>{{ config.module_name }}</span>
            </el-menu-item>
          </el-menu>
        </el-card>
      </el-col>

      <!-- 右侧：配置详情和编辑 -->
      <el-col :span="16">
        <el-card v-if="selectedConfig">
          <template #header>
            <div class="card-header">
              <span>{{ selectedConfig.module_name }} - 配置详情</span>
              <el-button type="primary" :loading="saving" @click="saveConfig">
                保存配置
              </el-button>
            </div>
          </template>

          <el-alert
              title="配置说明"
              type="info"
              :closable="false"
              style="margin-bottom: 16px"
          >
            此页面展示数据库中存储的模块配置。修改后将更新数据库中的配置值。
          </el-alert>

          <el-tabs v-model="activeTab">
            <!-- 配置值编辑 -->
            <el-tab-pane label="配置值" name="config">
              <div class="config-section">
                <h4>当前配置值:</h4>
                <SchemaFormInput
                    ref="formRef"
                    :schema="selectedConfig.json_schema"
                    :function-name="'config_value'"
                    :value="selectedConfig.config_value"
                />
              </div>
            </el-tab-pane>

            <!-- JSON Schema -->
            <el-tab-pane label="JSON Schema" name="schema">
              <h4>配置 JSON Schema:</h4>
              <el-input
                  v-model="schemaText"
                  :rows="15"
                  readonly
                  style="font-family: monospace"
                  type="textarea"
              />
            </el-tab-pane>

            <!-- 原始 JSON -->
            <el-tab-pane label="原始 JSON" name="raw">
              <h4>原始配置 JSON:</h4>
              <el-input
                  v-model="rawJsonText"
                  :rows="10"
                  type="textarea"
                  style="font-family: monospace"
              />
            </el-tab-pane>
          </el-tabs>

          <el-row style="margin-top: 20px" type="flex" justify="end">
            <el-button @click="resetForm">重置</el-button>
            <el-button type="primary" :loading="saving" @click="saveConfig">
              保存配置
            </el-button>
          </el-row>
        </el-card>

        <el-card v-else>
          <el-empty description="请选择一个模块配置"/>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script lang="ts" setup>
import {onMounted, ref, computed, watch} from 'vue'
import {ElMessage} from 'element-plus'
import {Setting} from '@element-plus/icons-vue'
import {
  listModuleConfigs,
  getModuleConfig,
  updateModuleConfig,
  type ModuleConfigSummary,
  type ModuleConfig,
} from '@/api/moduleConfigs'
import SchemaFormInput from '@/components/SchemaFormInput.vue'

const loading = ref(false)
const saving = ref(false)
const configs = ref<ModuleConfigSummary[]>([])
const selectedModuleName = ref<string>('')
const selectedConfig = ref<ModuleConfig | null>(null)
const activeTab = ref('config')
const formRef = ref<InstanceType<typeof SchemaFormInput> | null>(null)
const originalConfig = ref<Record<string, any>>({})

const schemaText = computed(() => {
  if (!selectedConfig.value?.json_schema) return '{}'
  return JSON.stringify(selectedConfig.value.json_schema, null, 2)
})

const rawJsonText = computed({
  get: () => {
    if (!selectedConfig.value?.config_value) return '{}'
    return JSON.stringify(selectedConfig.value.config_value, null, 2)
  },
  set: (val: string) => {
    try {
      if (selectedConfig.value) {
        selectedConfig.value.config_value = JSON.parse(val)
      }
    } catch {
      // ignore parse errors
    }
  }
})

// 加载配置列表
const loadConfigs = async () => {
  try {
    loading.value = true
    const response = await listModuleConfigs()
    configs.value = response.data.configs
  } catch (error) {
    ElMessage.error('加载配置列表失败')
  } finally {
    loading.value = false
  }
}

// 选择模块配置
const handleModuleSelect = async (key: string) => {
  selectedModuleName.value = key

  try {
    const response = await getModuleConfig(key)
    selectedConfig.value = response.data
    originalConfig.value = JSON.parse(JSON.stringify(response.data.config_value))
  } catch (error) {
    ElMessage.error('加载配置详情失败')
    selectedConfig.value = null
  }
}

// 保存配置
const saveConfig = async () => {
  if (!selectedConfig.value) {
    ElMessage.warning('请先选择一个配置')
    return
  }

  let configValue: Record<string, any>

  if (activeTab.value === 'config') {
    // 表单模式
    if (formRef.value) {
      const isValid = await formRef.value.validate()
      if (!isValid) {
        ElMessage.error('请检查表单输入')
        return
      }
      configValue = formRef.value.getFormData()
    } else {
      configValue = selectedConfig.value.config_value
    }
  } else {
    // JSON 模式
    try {
      configValue = JSON.parse(rawJsonText.value)
    } catch {
      ElMessage.error('JSON 格式错误')
      return
    }
  }

  try {
    saving.value = true
    await updateModuleConfig(selectedConfig.value.id, {
      config_value: configValue
    })
    ElMessage.success('配置保存成功')

    // 重新加载配置
    await handleModuleSelect(selectedModuleName.value)
  } catch (error) {
    ElMessage.error('保存配置失败')
  } finally {
    saving.value = false
  }
}

// 重置表单
const resetForm = () => {
  if (selectedConfig.value) {
    selectedConfig.value.config_value = JSON.parse(JSON.stringify(originalConfig.value))
  }
}

onMounted(() => {
  loadConfigs()
})

// 监听选中模块变化，重置表单引用
watch(selectedModuleName, () => {
  activeTab.value = 'config'
  formRef.value = null
})
</script>

<style lang="scss" scoped>
.module-config-manager {
  padding: 20px;

  .header-card {
    h2 {
      margin: 0 0 10px 0;
    }

    p {
      margin: 0;
      color: #666;
    }
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .config-section {
    h4 {
      margin: 0 0 10px 0;
    }
  }
}
</style>
