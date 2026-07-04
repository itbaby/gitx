/**
 * Wrap arbitrary text in a Markdown fenced code block whose fence is longer
 * than any run of backticks that occupies an entire line of `content`.
 *
 * Why: CommonMark closes a fenced code block at the first line containing only
 * backticks of length >= the opening fence. Git diffs of Markdown / code files
 * routinely contain bare ``` lines (unchanged context lines that are fences in
 * the source), so a fixed 3-backtick wrapper can be terminated prematurely and
 * everything after it leaks out of the code block.
 *
 * Picking a fence strictly longer than any all-backtick line guarantees the
 * block stays open until our explicit closing fence.
 */
export const wrapCodeBlock = (content: string): string => {
  // Only a line of >= 3 backticks can act as a closing fence (CommonMark
  // requires at least 3), so shorter all-backtick lines are not a threat.
  let maxFence = 0
  for (const line of content.split('\n')) {
    const trimmed = line.trim()
    if (trimmed.length >= 3 && /^[`]+$/.test(trimmed)) {
      if (trimmed.length > maxFence) maxFence = trimmed.length
    }
  }
  // Default to 3 (a standard code fence); exceed the longest threat if any.
  const fence = '`'.repeat(Math.max(3, maxFence + 1))
  return `${fence}\n${content}\n${fence}`
}

/**
 * Format an agent tool result for inline display in a chat message: a bold
 * header naming the tool, followed by the result in a fenced code block, all
 * wrapped in a single blockquote so it's visually distinguished from the
 * assistant's prose. The code fence is dynamically sized to survive fence
 * characters appearing inside the (arbitrary) tool output.
 */
export const formatToolResult = (name: string, result: string): string => {
  const section = `**${name}**\n${wrapCodeBlock(result)}`
  // Prefix every line (including blanks) with "> " so the whole section forms
  // one contiguous blockquote and the embedded code fence parses correctly.
  return `\n\n${section.replace(/^/gm, '> ')}\n`
}
