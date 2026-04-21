<script setup lang="ts">
import { ref, nextTick, watch } from 'vue'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import type { ChatMessage } from '../types'

const props = defineProps<{
  messages: ChatMessage[]
  hasDiff: boolean
  loading: boolean
  isStreaming: boolean
  toolStatus: string
}>()

const emit = defineEmits<{
  send: [text: string]
  clear: []
}>()

const inputText = ref('')
const messagesContainer = ref<HTMLElement | null>(null)

// 自动滚动到底部
const scrollToBottom = async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

watch(() => props.messages.length, scrollToBottom)
watch(() => {
  const lastMsg = props.messages[props.messages.length - 1]
  return lastMsg?.content?.length
}, scrollToBottom)

const handleSend = () => {
  const text = inputText.value.trim()
  if (!text) return
  emit('send', text)
  inputText.value = ''
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}

const autoResize = (e: Event) => {
  const target = e.target as HTMLTextAreaElement
  target.style.height = 'auto'
  target.style.height = Math.min(target.scrollHeight, 120) + 'px'
}

const quickPrompts = [
  '分析这些变更的影响',
  '有什么潜在问题？',
  '代码质量评估',
]

const handleQuickPrompt = (prompt: string) => {
  emit('send', prompt)
}

// Markdown rendering with marked + DOMPurify
const renderMarkdown = (text: string): string => {
  if (!text) return ''
  const rawHtml = marked.parse(text, { async: false }) as string
  return DOMPurify.sanitize(rawHtml)
}

// Escape HTML for user messages (prevent XSS)
const escapeHtml = (text: string): string => {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}
</script>

<template>
  <aside class="ai-panel">
    <!-- 面板标题 -->
    <div class="panel-header">
      <div class="panel-title">
        <svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor" class="ai-icon">
          <path d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm6.5-.25A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z"/>
        </svg>
        <span>AI 助手</span>
      </div>
      <button v-if="messages.length > 0" class="btn btn-ghost btn-sm" @click="emit('clear')" title="清除对话">
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M11 1.75v5h1.25a.25.25 0 0 1 .177.427l-3.25 3.25a.25.25 0 0 1-.354 0l-3.25-3.25A.25.25 0 0 1 6.75 6.75H8V1.75a.25.25 0 0 1 .25-.25h2.5a.25.25 0 0 1 .25.25Z"/>
          <path d="M1.5 12.75a.25.25 0 0 1 .25-.25h4.5a.25.25 0 0 1 .25.25v1.5a.25.25 0 0 1-.25.25h-4.5a.25.25 0 0 1-.25-.25Z"/>
        </svg>
      </button>
    </div>

    <!-- 快捷提示 -->
    <div v-if="messages.length === 0 && hasDiff" class="quick-prompts">
      <span class="quick-label">快捷操作</span>
      <button
        v-for="prompt in quickPrompts"
        :key="prompt"
        class="quick-btn"
        @click="handleQuickPrompt(prompt)"
      >
        {{ prompt }}
      </button>
    </div>

    <!-- 工具执行状态 -->
    <div v-if="toolStatus" class="tool-status">
      <span class="tool-spinner"></span>
      <span>{{ toolStatus }}</span>
    </div>

    <!-- 消息列表 -->
    <div ref="messagesContainer" class="messages-container">
      <!-- 欢迎消息 -->
      <div v-if="messages.length === 0" class="welcome-message">
        <div class="welcome-icon">✨</div>
        <p>你可以：</p>
        <ul>
          <li>输入自然语言命令，如「比较当前分支和 main」</li>
          <li>比较分支后，输入「分析这些变更」</li>
          <li>以 <code>/</code> 开头输入分析指令</li>
        </ul>
      </div>

      <!-- 消息气泡 -->
      <div
        v-for="msg in messages"
        :key="msg.id"
        class="message"
        :class="[msg.role, { streaming: msg.isStreaming }]"
      >
        <div class="message-avatar" v-if="msg.role === 'user'">
          <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
            <path d="M10.561 8.073a6.005 6.005 0 0 1 3.432 5.142.75.75 0 1 1-1.498.07 4.5 4.5 0 0 0-8.99 0 .75.75 0 0 1-1.498-.07 6.005 6.005 0 0 1 3.432-5.142 3.999 3.999 0 1 1 5.123 0ZM10.5 5a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0Z"/>
          </svg>
        </div>
        <div class="message-content" v-html="msg.role === 'user' ? escapeHtml(msg.content) : renderMarkdown(msg.content)"></div>
        <div class="message-avatar ai-avatar" v-if="msg.role === 'assistant'">
          <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
            <path d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13Z"/>
          </svg>
        </div>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="input-area">
      <div class="input-wrapper">
        <textarea
          v-model="inputText"
          placeholder="输入消息... (Enter 发送)"
          rows="1"
          :disabled="loading || isStreaming"
          @keydown="handleKeyDown"
          @input="autoResize($event)"
        ></textarea>
        <button
          class="btn btn-primary send-btn"
          :disabled="loading || isStreaming || !inputText.trim()"
          @click="handleSend"
        >
          <svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor">
            <path d="M1.5 1.75a.75.75 0 0 1 1.06-.06l10.5 9a.75.75 0 0 1-.36 1.37H9.06l-2.72 4.08a.75.75 0 0 1-1.22-.24L2.39 9.31 1.06 8.56a.75.75 0 0 1 .44-1.31Z"/>
          </svg>
        </button>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.ai-panel {
  width: var(--ai-panel-width);
  min-width: var(--ai-panel-width);
  background-color: var(--bg-secondary);
  border-left: 1px solid var(--border-default);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

/* 面板标题 */
.panel-header {
  padding: var(--space-sm) var(--space-md);
  border-bottom: 1px solid var(--border-default);
  flex-shrink: 0;
}

.panel-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.ai-icon {
  color: var(--accent-default);
}

/* 快捷提示 */
.quick-prompts {
  padding: var(--space-md);
  border-bottom: 1px solid var(--border-muted);
}

.quick-label {
  display: block;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-bottom: var(--space-sm);
}

.quick-btn {
  display: block;
  width: 100%;
  text-align: left;
  padding: var(--space-sm) var(--space-md);
  margin-bottom: var(--space-xs);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  background-color: var(--bg-tertiary);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.quick-btn:hover {
  color: var(--accent-default);
  border-color: var(--accent-muted);
  background-color: var(--accent-subtle);
}

/* 消息列表 */
.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

/* 工具执行状态 */
.tool-status {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  font-size: var(--text-sm);
  color: var(--accent-default);
  background-color: var(--accent-subtle);
  border-bottom: 1px solid var(--border-muted);
  flex-shrink: 0;
}

.tool-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--accent-muted);
  border-top-color: var(--accent-default);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 欢迎消息 */
.welcome-message {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  padding: var(--space-2xl) var(--space-lg);
}

.welcome-icon {
  font-size: 32px;
  margin-bottom: var(--space-md);
}

.welcome-message p {
  margin-bottom: var(--space-sm);
  color: var(--text-secondary);
}

.welcome-message ul {
  list-style: none;
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.welcome-message li {
  padding-left: var(--space-lg);
  position: relative;
}

.welcome-message li::before {
  content: '•';
  position: absolute;
  left: var(--space-sm);
  color: var(--accent-default);
}

.welcome-message code {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--accent-default);
  background-color: var(--accent-subtle);
  padding: 1px 4px;
  border-radius: var(--radius-sm);
}

/* 消息气泡 */
.message {
  display: flex;
  gap: var(--space-sm);
  animation: fadeIn 0.2s ease;
}

.message.user {
  flex-direction: row-reverse;
}

.message-avatar {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
}

.message.user .message-avatar {
  background-color: var(--accent-subtle);
  color: var(--accent-default);
}

.ai-avatar {
  background-color: var(--accent-subtle);
  color: var(--accent-default);
}

.message-content {
  max-width: calc(100% - 40px);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-lg);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
  word-wrap: break-word;
  overflow-wrap: break-word;
}

.message.assistant .message-content {
  background-color: var(--bg-tertiary);
  color: var(--text-primary);
  border-top-left-radius: var(--radius-sm);
}

.message.user .message-content {
  background-color: var(--accent-default);
  color: var(--text-on-accent);
  border-top-right-radius: var(--radius-sm);
}

.message.streaming .message-content::after {
  content: '▊';
  animation: blink 1s steps(2) infinite;
  margin-left: 2px;
}

@keyframes blink {
  50% { opacity: 0; }
}

/* Markdown 渲染样式 */
.message-content :deep(h2),
.message-content :deep(h3),
.message-content :deep(h4) {
  color: var(--text-primary);
  margin: var(--space-sm) 0 var(--space-xs);
}

.message-content :deep(code) {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  background-color: var(--bg-primary);
  padding: 1px 4px;
  border-radius: 3px;
}

.message-content :deep(pre) {
  margin: var(--space-sm) 0;
  background-color: var(--bg-primary);
  border-radius: var(--radius-sm);
  padding: var(--space-sm);
  overflow-x: auto;
}

.message-content :deep(pre code) {
  background: transparent;
  padding: 0;
}

.message-content :deep(ul) {
  margin: var(--space-xs) 0;
  padding-left: var(--space-lg);
}

.message-content :deep(li) {
  margin-bottom: 2px;
}

/* 输入区域 */
.input-area {
  padding: var(--space-sm) var(--space-md) var(--space-md);
  border-top: 1px solid var(--border-default);
  flex-shrink: 0;
}

.input-wrapper {
  display: flex;
  align-items: flex-end;
  gap: var(--space-sm);
  background-color: var(--bg-primary);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  padding: var(--space-xs) var(--space-xs) var(--space-xs) var(--space-md);
  transition: border-color var(--transition-fast);
}

.input-wrapper:focus-within {
  border-color: var(--accent-default);
}

.input-wrapper textarea {
  flex: 1;
  border: none;
  background: transparent;
  padding: var(--space-sm) 0;
  resize: none;
  max-height: 120px;
  line-height: 1.4;
}

.input-wrapper textarea:focus {
  box-shadow: none;
}

.send-btn {
  width: 32px;
  height: 32px;
  min-width: 32px;
  padding: 0;
  border-radius: var(--radius-md);
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(4px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
