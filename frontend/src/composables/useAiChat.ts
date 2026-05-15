import { ref } from 'vue'
import { aiApi } from '../api/index'
import type { ChatMessage, InputMessage, ChatContext, DiffInfo } from '../types/index'
import { friendlyError } from '../utils/error'

const MAX_AI_MESSAGES = 100

export function useAiChat(
  getBaseBranch: () => string,
  getCompareBranch: () => string,
  getDiffData: () => DiffInfo[],
) {
  const aiMessages = ref<ChatMessage[]>([])
  const toolStatus = ref('')
  const isStreaming = ref(false)

  const handleChat = async (text: string) => {
    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: text,
      timestamp: new Date(),
    }
    aiMessages.value.push(userMsg)

    const aiMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'assistant',
      content: '',
      timestamp: new Date(),
      isStreaming: true,
    }
    aiMessages.value.push(aiMsg)
    isStreaming.value = true

    // Trim old messages to prevent unbounded growth
    if (aiMessages.value.length > MAX_AI_MESSAGES) {
      aiMessages.value = aiMessages.value.slice(-MAX_AI_MESSAGES)
    }

    // Build chat history from non-streaming messages only (for token efficiency)
    const historyMessages = aiMessages.value.filter(
      (m) => m.role === 'user' || (m.role === 'assistant' && !m.isStreaming),
    )
    const chatHistory: InputMessage[] = historyMessages.slice(-20).map(
      (m) => ({ role: m.role as 'user' | 'assistant', content: m.content }),
    )

    const chatCtx: ChatContext = {
      base_branch: getBaseBranch(),
      compare_branch: getCompareBranch(),
      has_diff: getDiffData().length > 0,
    }

    try {
      await aiApi.chat(
        chatHistory,
        chatCtx,
        (_name, display) => { toolStatus.value = display },
          (name, result) => {
            // Show tool result in chat so users see what data the agent retrieved.
            toolStatus.value = ''
            aiMsg.content += `\n\n> **${name}**\n> \`\`\`\n> ${result.replace(/\n/g, '\n> ')}\n> \`\`\`\n`
          },
        (chunk) => {
          toolStatus.value = ''
          aiMsg.content += chunk
        },
        () => {
          aiMsg.isStreaming = false
          isStreaming.value = false
        },
        (err) => {
          aiMsg.content = `请求失败: ${friendlyError(err)}`
          aiMsg.isStreaming = false
          isStreaming.value = false
          toolStatus.value = ''
        },
      )
    } catch (err: unknown) {
      aiMsg.content = `请求失败: ${friendlyError(err)}`
      aiMsg.isStreaming = false
      isStreaming.value = false
      toolStatus.value = ''
    }
  }

  const handleAnalyze = async (prompt: string) => {
    const diffData = getDiffData()
    if (diffData.length === 0) return

    const userMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: prompt,
      timestamp: new Date(),
    }
    aiMessages.value.push(userMsg)

    const aiMsg: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'assistant',
      content: '',
      timestamp: new Date(),
      isStreaming: true,
    }
    aiMessages.value.push(aiMsg)
    isStreaming.value = true

    try {
      await aiApi.analyzeStream(
        diffData,
        prompt,
        (chunk) => { aiMsg.content += chunk },
        () => {
          aiMsg.isStreaming = false
          isStreaming.value = false
        },
        (err) => {
          aiMsg.content = `请求失败: ${friendlyError(err)}`
          aiMsg.isStreaming = false
          isStreaming.value = false
        },
      )
    } catch (err: unknown) {
      aiMsg.content = `请求失败: ${friendlyError(err)}`
      aiMsg.isStreaming = false
      isStreaming.value = false
    }
  }

  const clearChat = () => {
    aiMessages.value = []
    toolStatus.value = ''
    isStreaming.value = false
  }

  return {
    aiMessages,
    toolStatus,
    isStreaming,
    handleChat,
    handleAnalyze,
    clearChat,
  }
}
