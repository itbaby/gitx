/** Map raw errors to user-friendly messages using regex patterns. */
export const friendlyError = (err: unknown): string => {
  if (!(err instanceof Error)) return String(err)
  const msg = err.message

  const patterns: Array<{ re: RegExp; text: string }> = [
    { re: /AI 客户端未初始化/, text: 'AI 功能未配置，请检查 .env 文件中的 OPENAI_API_KEY' },
    { re: /AI API 错误 \(401\)/, text: 'API Key 无效，请检查配置' },
    { re: /AI API 错误 \(429\)/, text: '请求过于频繁，请稍后再试' },
    { re: /AI API 错误/, text: 'AI 服务暂时不可用，请稍后再试' },
    { re: /AI API 请求失败/, text: '无法连接到 AI 服务，请检查网络' },
  ]

  for (const { re, text } of patterns) {
    if (re.test(msg)) return text
  }

  return msg
}
