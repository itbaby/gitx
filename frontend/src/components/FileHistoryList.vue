<script setup lang="ts">
import type { CommitInfo } from '../types'
import { formatTimestamp, formatFullDate } from '../utils/date'

defineProps<{
  commits: CommitInfo[]
  file: string
}>()
</script>

<template>
  <div class="file-history">
    <div v-if="commits.length === 0" class="empty-state">
      <p v-if="!file">在上方输入文件路径查询历史变更</p>
      <p v-else>在所选时间范围内未找到 <code>{{ file }}</code> 的变更记录</p>
    </div>

    <div v-else class="history-list">
      <div class="history-header">
        <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
          <path d="M2 1.75C2 .784 2.784 0 3.75 0h6.586c.464 0 .909.184 1.237.513l2.914 2.914c.329.328.513.773.513 1.237v9.586A1.75 1.75 0 0 1 13.25 16h-9.5A1.75 1.75 0 0 1 2 14.25Zm1.75-.25a.25.25 0 0 0-.25.25v12.5c0 .138.112.25.25.25h9.5a.25.25 0 0 0 .25-.25V6h-2.75A1.75 1.75 0 0 1 8.75 4.25V1.5Zm6.75.062V4.25c0 .138.112.25.25.25h2.688l-.011-.013-2.914-2.914-.013-.011Z"/>
        </svg>
        <span class="file-path" v-if="file">{{ file }}</span>
        <span class="history-count">共 {{ commits.length }} 条变更记录</span>
      </div>

      <div v-for="commit in commits" :key="commit.hash" class="history-item">
        <div class="history-item-left">
          <span class="commit-hash">{{ commit.shortHash }}</span>
          <span class="commit-author">{{ commit.author }}</span>
          <span class="commit-time" :title="formatFullDate(commit.timestamp)">{{ formatTimestamp(commit.timestamp) }}</span>
        </div>
        <div class="history-item-right">
          <span class="commit-message">{{ commit.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-history {
  animation: fadeIn 0.2s ease;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-tertiary);
  gap: var(--space-sm);
}

.empty-state code {
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  color: var(--accent-default);
  background-color: var(--accent-subtle);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  background-color: var(--border-muted);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.history-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  background-color: var(--bg-tertiary);
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.file-path {
  font-family: var(--font-mono);
  color: var(--color-info);
  font-weight: 500;
}

.history-count {
  margin-left: auto;
  color: var(--text-tertiary);
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-sm) var(--space-md);
  background-color: var(--bg-secondary);
  transition: background-color var(--transition-fast);
}

.history-item:hover {
  background-color: var(--bg-tertiary);
}

.history-item-left {
  display: flex;
  align-items: center;
  gap: var(--space-md);
  flex-shrink: 0;
}

.commit-hash {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--accent-default);
}

.commit-author {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.commit-time {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.history-item-right {
  flex: 1;
  min-width: 0;
  text-align: right;
}

.commit-message {
  font-size: var(--text-sm);
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
