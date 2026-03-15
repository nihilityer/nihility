<template>
  <div class="schema-form-input">
    <VueForm
      v-model="formData"
      :schema="normalizedSchema"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import VueForm from '@lljj/vue3-form-element'

interface Props {
  schema: any
  functionName: string
}

const props = defineProps<Props>()

const formData = ref<any>({})

// Schema 规范化处理
const normalizedSchema = computed(() => {
  if (!props.schema) return {}

  const schema = JSON.parse(JSON.stringify(props.schema))

  // 处理 nullable 类型
  if (schema.properties) {
    Object.keys(schema.properties).forEach((key) => {
      const prop = schema.properties[key]
      if (Array.isArray(prop.type) && prop.type.includes('null')) {
        // 提取非 null 类型
        const nonNullTypes = prop.type.filter((t: string) => t !== 'null')
        prop.type = nonNullTypes.length > 0 ? nonNullTypes[0] : 'string'
      }
    })
  }

  return schema
})

// 初始化默认值
watch(
  () => props.schema,
  (newSchema) => {
    if (newSchema && newSchema.properties) {
      const defaults: any = {}
      Object.keys(newSchema.properties).forEach((key) => {
        const prop = newSchema.properties[key]
        if (prop.default !== undefined) {
          defaults[key] = prop.default
        }
      })
      formData.value = defaults
    }
  },
  { immediate: true }
)

// 暴露方法给父组件
const getFormData = () => {
  return formData.value
}

const validate = async (): Promise<boolean> => {
  // VueForm 自动验证，这里简单检查必填字段
  if (!normalizedSchema.value.required) {
    return true
  }

  const required = normalizedSchema.value.required as string[]
  for (const field of required) {
    if (!formData.value[field]) {
      return false
    }
  }

  return true
}

const reset = () => {
  formData.value = {}
}

defineExpose({
  getFormData,
  validate,
  reset,
})
</script>

<style scoped lang="scss">
.schema-form-input {
  :deep(.el-form-item) {
    margin-bottom: 18px;
  }

  :deep(.el-form-item__label) {
    font-weight: 500;
  }
}
</style>
