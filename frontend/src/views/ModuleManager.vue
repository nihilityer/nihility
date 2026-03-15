<template>
  <div class="module-manager">
    <el-card class="header-card">
      <h2>模块管理</h2>
      <p>管理和调用系统中加载的各个功能模块</p>
    </el-card>

    <el-row :gutter="20" style="margin-top: 20px">
      <!-- 左侧：模块列表 -->
      <el-col :span="8">
        <el-card>
          <template #header>
            <div class="card-header">
              <span>已加载模块</span>
              <el-button :loading="loading" size="small" type="primary" @click="loadModules">
                刷新
              </el-button>
            </div>
          </template>
          <el-menu :default-active="selectedModule" @select="handleModuleSelect">
            <el-menu-item
                v-for="(module, index) in modules"
                :key="index"
                :index="getModuleKey(module)"
            >
              <el-icon>
                <Grid/>
              </el-icon>
              <span>{{ getModuleName(module) }}</span>
            </el-menu-item>
          </el-menu>
        </el-card>
      </el-col>

      <!-- 右侧：功能列表和调用界面 -->
      <el-col :span="16">
        <el-card v-if="selectedModule">
          <template #header>
            <div class="card-header">
              <span>{{ getModuleName(currentModuleType) }} - 功能列表</span>
            </div>
          </template>

          <el-tabs v-model="activeTab">
            <!-- 低权限功能 -->
            <el-tab-pane label="低权限功能" name="no_perm">
              <div v-if="functions?.no_perm_func.length">
                <el-collapse v-model="activeFunction">
                  <el-collapse-item
                      v-for="func in functions.no_perm_func"
                      :key="func.name"
                      :name="func.name"
                  >
                    <template #title>
                      <div class="function-title">
                        <el-tag size="small">{{ func.name }}</el-tag>
                        <span style="margin-left: 10px">{{ func.desc }}</span>
                      </div>
                    </template>
                    <div class="function-detail">
                      <div class="tags">
                        <el-tag v-for="tag in func.tags" :key="tag" size="small" type="info">
                          {{ tag }}
                        </el-tag>
                      </div>
                      <div class="params">
                        <h4>参数定义:</h4>
                        <el-input
                            v-model="paramInputs[func.name]"
                            :rows="4"
                            placeholder="请输入 JSON 格式的参数"
                            type="textarea"
                        />
                      </div>
                      <div class="actions">
                        <el-button type="primary" @click="callFunction(func.name, false)">
                          调用方法
                        </el-button>
                      </div>
                    </div>
                  </el-collapse-item>
                </el-collapse>
              </div>
              <el-empty v-else description="暂无低权限功能"/>
            </el-tab-pane>

            <!-- 高权限功能 -->
            <el-tab-pane label="高权限功能" name="perm">
              <div v-if="functions?.perm_func.length">
                <el-collapse v-model="activeFunction">
                  <el-collapse-item
                      v-for="func in functions.perm_func"
                      :key="func.name"
                      :name="func.name"
                  >
                    <template #title>
                      <div class="function-title">
                        <el-tag size="small" type="danger">{{ func.name }}</el-tag>
                        <span style="margin-left: 10px">{{ func.desc }}</span>
                      </div>
                    </template>
                    <div class="function-detail">
                      <div class="tags">
                        <el-tag v-for="tag in func.tags" :key="tag" size="small" type="info">
                          {{ tag }}
                        </el-tag>
                      </div>
                      <div class="params">
                        <h4>参数定义:</h4>
                        <el-input
                            v-model="paramInputs[func.name]"
                            :rows="4"
                            placeholder="请输入 JSON 格式的参数"
                            type="textarea"
                        />
                      </div>
                      <div class="actions">
                        <el-button type="danger" @click="callFunction(func.name, true)">
                          调用方法（可变）
                        </el-button>
                      </div>
                    </div>
                  </el-collapse-item>
                </el-collapse>
              </div>
              <el-empty v-else description="暂无高权限功能"/>
            </el-tab-pane>

            <!-- 调用结果 -->
            <el-tab-pane label="调用结果" name="result">
              <div v-if="callResult">
                <h4>调用结果:</h4>
                <el-input
                    v-model="callResult"
                    :rows="10"
                    readonly
                    style="font-family: monospace"
                    type="textarea"
                />
              </div>
              <el-empty v-else description="暂无调用结果"/>
            </el-tab-pane>
          </el-tabs>
        </el-card>

        <el-card v-else>
          <el-empty description="请选择一个模块"/>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script lang="ts" setup>
import {onMounted, ref, watch} from 'vue'
import {ElMessage} from 'element-plus'
import {Grid} from '@element-plus/icons-vue'
import {
  callModuleFunction,
  getLoadedModules,
  type ModuleFunctions,
  type ModuleType,
  queryModuleFunctions,
} from '@/api/modules'

const loading = ref(false)
const modules = ref<ModuleType[]>([])
const selectedModule = ref<string>('')
const currentModuleType = ref<ModuleType | null>(null)
const functions = ref<ModuleFunctions | null>(null)
const activeTab = ref('no_perm')
const activeFunction = ref<string[]>([])
const paramInputs = ref<Record<string, string>>({})
const callResult = ref<string>('')

// 加载模块列表
const loadModules = async () => {
  try {
    loading.value = true
    const response = await getLoadedModules()
    modules.value = response.data
  } catch (error) {
    ElMessage.error('加载模块列表失败')
  } finally {
    loading.value = false
  }
}

// 获取模块键值
const getModuleKey = (module: ModuleType): string => {
  if (module.Embed === 'BrowserControl') return 'browser_control'
  if (module.Embed === 'EdgeDeviceControl') return 'edge_device_control'
  if (module.Wasm) return `wasm_${module.Wasm}`
  return 'unknown'
}

// 获取模块名称
const getModuleName = (module: ModuleType | null): string => {
  if (!module) return ''
  if (module.Embed === 'BrowserControl') return '浏览器控制'
  if (module.Embed === 'EdgeDeviceControl') return '边缘设备控制'
  if (module.Wasm) return `WASM 模块: ${module.Wasm}`
  return '未知模块'
}

// 选择模块
const handleModuleSelect = async (key: string) => {
  selectedModule.value = key
  const module = modules.value.find((m) => getModuleKey(m) === key)
  if (!module) return

  currentModuleType.value = module

  try {
    const response = await queryModuleFunctions(key)
    functions.value = response.data
  } catch (error) {
    ElMessage.error('加载模块功能失败')
  }
}

// 调用功能
const callFunction = async (funcName: string, isMut: boolean) => {
  if (!selectedModule.value) {
    ElMessage.warning('请先选择模块')
    return
  }

  let param: any
  try {
    const paramStr = paramInputs.value[funcName] || '{}'
    param = JSON.parse(paramStr)
  } catch (error) {
    ElMessage.error('参数格式错误，请输入有效的 JSON')
    return
  }

  try {
    const response = await callModuleFunction(selectedModule.value, {
      func_name: funcName,
      param,
      is_mut: isMut,
    })
    callResult.value = JSON.stringify(response.data.result, null, 2)
    activeTab.value = 'result'
    ElMessage.success('调用成功')
  } catch (error) {
    ElMessage.error('调用失败')
  }
}

onMounted(() => {
  loadModules()
})

// 监听选中的模块变化，清空调用结果
watch(selectedModule, () => {
  callResult.value = ''
  activeTab.value = 'no_perm'
})
</script>

<style lang="scss" scoped>
.module-manager {
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

  .function-title {
    display: flex;
    align-items: center;
    width: 100%;
  }

  .function-detail {
    padding: 10px;

    .tags {
      margin-bottom: 15px;
      display: flex;
      gap: 8px;
    }

    .params {
      margin-bottom: 15px;

      h4 {
        margin: 0 0 10px 0;
      }
    }

    .actions {
      display: flex;
      justify-content: flex-end;
    }
  }
}
</style>
