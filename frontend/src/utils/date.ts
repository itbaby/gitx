export const formatTimestamp = (ts: string): string => {
  const date = new Date(ts)
  if (isNaN(date.getTime())) return '未知时间'

  const now = new Date()
  const diffMs = now.getTime() - date.getTime()

  // Future timestamps (clock skew) — treat as "just now"
  if (diffMs < 0) return '刚刚'

  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return '刚刚'
  if (diffMins < 60) return `${diffMins} 分钟前`
  if (diffHours < 24) return `${diffHours} 小时前`
  if (diffDays < 30) return `${diffDays} 天前`

  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}

export const formatFullDate = (ts: string): string => {
  const date = new Date(ts)
  if (isNaN(date.getTime())) return '未知时间'
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}
