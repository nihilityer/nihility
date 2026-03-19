<template>
  <div class="html-editor">
    <div class="editor-toolbar">
      <el-button-group size="small">
        <el-button
            :type="!isDarkTheme ? 'primary' : ''"
            title="亮色主题"
            @click="setTheme(false)"
        >
          <el-icon>
            <Sunny/>
          </el-icon>
        </el-button>
        <el-button
            :type="isDarkTheme ? 'primary' : ''"
            title="暗色主题"
            @click="setTheme(true)"
        >
          <el-icon>
            <Moon/>
          </el-icon>
        </el-button>
      </el-button-group>
    </div>
    <div ref="editorContainer" class="editor-container"></div>
    <div class="editor-footer">
      <span class="stats">
        {{ contentLength }} 字符 | {{ lineCount }} 行
      </span>
    </div>
  </div>
</template>

<script lang="ts" setup>
import {computed, onBeforeUnmount, onMounted, ref, watch} from 'vue'
import {Moon, Sunny} from '@element-plus/icons-vue'
import {EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightActiveLine} from '@codemirror/view'
import {EditorState, Compartment} from '@codemirror/state'
import {defaultKeymap, history, historyKeymap} from '@codemirror/commands'
import {html} from '@codemirror/lang-html'
import {autocompletion, closeBrackets, closeBracketsKeymap} from '@codemirror/autocomplete'
import {
  foldGutter,
  foldKeymap,
  indentOnInput,
  syntaxHighlighting,
  defaultHighlightStyle
} from '@codemirror/language'
import {oneDark} from '@codemirror/theme-one-dark'
import {searchKeymap} from '@codemirror/search'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const editorContainer = ref<HTMLElement | null>(null)
const editorView = ref<EditorView | null>(null)
const isDarkTheme = ref(false)
const themeCompartment = new Compartment()
let isInternalUpdate = false

const contentLength = computed(() => editorView.value?.state.doc.length ?? 0)
const lineCount = computed(() => editorView.value?.state.doc.lines ?? 0)

// 自定义亮色主题
const customLightTheme = EditorView.theme({
  '&': {
    fontSize: '14px',
    fontFamily: "'Courier New', Courier, monospace",
    backgroundColor: '#ffffff',
    height: '100%'
  },
  '.cm-content': {
    caretColor: 'var(--el-color-primary)',
    padding: '8px 0'
  },
  '.cm-line': {
    padding: '0 8px',
    lineHeight: '1.5'
  },
  '.cm-gutters': {
    backgroundColor: 'var(--el-fill-color-light)',
    color: 'var(--el-text-color-secondary)',
    border: 'none'
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--el-fill-color)'
  },
  '.cm-activeLine': {
    backgroundColor: 'var(--el-fill-color-lighter)'
  },
  '.cm-selectionBackground': {
    backgroundColor: 'var(--el-color-primary-light-9) !important'
  },
  '&.cm-focused .cm-selectionBackground': {
    backgroundColor: 'var(--el-color-primary-light-8) !important'
  },
  '.cm-foldGutter': {
    width: '16px'
  }
}, {dark: false})

// 初始化编辑器
onMounted(() => {
  if (!editorContainer.value) return

  const startState = EditorState.create({
    doc: props.modelValue,
    extensions: [
      lineNumbers(),
      highlightActiveLineGutter(),
      highlightActiveLine(),
      history(),
      foldGutter({
        openText: '▼',
        closedText: '▶'
      }),
      indentOnInput(),
      syntaxHighlighting(defaultHighlightStyle),
      closeBrackets(),
      autocompletion({
        activateOnTyping: true
      }),
      html({
        autoCloseTags: true,
        matchClosingTags: true
      }),
      keymap.of([
        ...defaultKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...closeBracketsKeymap,
        ...searchKeymap
      ]),
      themeCompartment.of(customLightTheme),
      EditorView.updateListener.of((update) => {
        if (update.docChanged && !isInternalUpdate) {
          emit('update:modelValue', update.state.doc.toString())
        }
      })
    ]
  })

  editorView.value = new EditorView({
    state: startState,
    parent: editorContainer.value
  })
})

// 监听外部变化
watch(() => props.modelValue, (newValue) => {
  if (editorView.value && editorView.value.state.doc.toString() !== newValue) {
    isInternalUpdate = true
    editorView.value.dispatch({
      changes: {
        from: 0,
        to: editorView.value.state.doc.length,
        insert: newValue
      }
    })
    isInternalUpdate = false
  }
})

// 主题切换
const setTheme = (dark: boolean) => {
  isDarkTheme.value = dark
  editorView.value?.dispatch({
    effects: themeCompartment.reconfigure(
        dark ? oneDark : customLightTheme
    )
  })
}

// 清理
onBeforeUnmount(() => {
  editorView.value?.destroy()
})
</script>

<style scoped>
.html-editor {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  overflow: hidden;
}

.editor-toolbar {
  display: flex;
  justify-content: flex-end;
  padding: 8px;
  background-color: var(--el-fill-color-light);
  border-bottom: 1px solid var(--el-border-color);
}

.editor-container {
  flex: 1;
  min-height: 400px;
  max-height: 600px;
  overflow: auto;
}

.editor-footer {
  display: flex;
  justify-content: flex-end;
  padding: 4px 8px;
  background-color: var(--el-fill-color-light);
  border-top: 1px solid var(--el-border-color);
}

.stats {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>

