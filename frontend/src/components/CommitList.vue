<script setup lang="ts">
import type { CommitInfo } from '../types'

defineProps<{
  commits: CommitInfo[]
}>()

defineEmits<{
  refresh: []
}>()

const formatTimestamp = (ts: string) => {
  const date = new Date(ts)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return '刚刚'
  if (diffMins < 60) return `${diffMins} 分钟前`
  if (diffHours < 24) return `${diffHours} 小时前`
  if (diffDays < 30) return `${diffDays} 天前`

  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}

const formatFullDate = (ts: string) => {
  return new Date(ts).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div class="commit-list">
    <div v-if="commits.length === 0" class="empty-state">
      <p>暂无提交记录</p>
    </div>

    <div v-else class="commit-timeline">
      <div v-for="commit in commits" :key="commit.hash" class="commit-item">
        <div class="commit-indicator">
          <div class="commit-dot"></div>
          <div class="commit-line"></div>
        </div>
        <div class="commit-body">
          <div class="commit-header">
            <span class="commit-hash" :title="commit.hash">{{ commit.shortHash }}</span>
            <span class="commit-message">{{ commit.message }}</span>
          </div>
          <div class="commit-meta">
            <span class="commit-author">
              <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                <path d="M10.561 8.073a6.005 6.005 0 0 1 3.432 5.142.75.75 0 1 1-1.498.07 4.5 4.5 0 0 0-8.99 0 .75.75 0 0 1-1.498-.07 6.005 6.005 0 0 1 3.432-5.142 3.999 3.999 0 1 1 5.123 0ZM10.5 5a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0Z"/>
              </svg>
              {{ commit.author }}
            </span>
            <span class="commit-time" :title="formatFullDate(commit.timestamp)">
              <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
                <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Zm10-3H8a.5.5 0 0 0-.5.5v3.793L5.354 10.94a.5.5 0 1 1-.708-.708l2.5-2.5A.5.5 0 0 1 8 7.5h3.5a.5.5 0 0 1 0 1Z"/>
              </svg>
              {{ formatTimestamp(commit.timestamp) }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.commit-list {
  animation: fadeIn 0.2s ease;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-tertiary);
}

.commit-timeline {
  display: flex;
  flex-direction: column;
}

.commit-item {
  display: flex;
  gap: var(--space-md);
  position: relative;
}

.commit-item:last-child .commit-line {
  display: none;
}

.commit-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 24px;
  flex-shrink: 0;
}

.commit-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: var(--accent-default);
  border: 2px solid var(--bg-primary);
  margin-top: 6px;
  flex-shrink: 0;
  z-index: 1;
}

.commit-line {
  width: 2px;
  flex: 1;
  background-color: var(--border-default);
  margin-top: var(--space-xs);
}

.commit-body {
  flex: 1;
  padding-bottom: var(--space-lg);
  min-width: 0;
}

.commit-header {
  display: flex;
  align-items: baseline;
  gap: var(--space-sm);
  margin-bottom: var(--space-xs);
}

.commit-hash {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--accent-default);
  background-color: var(--accent-subtle);
  padding: 1px var(--space-sm);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.commit-message {
  font-size: var(--text-sm);
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-meta {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.commit-author,
.commit-time {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
