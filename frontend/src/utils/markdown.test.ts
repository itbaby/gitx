import { describe, it, expect } from 'vitest'
import { marked } from 'marked'
import { wrapCodeBlock, formatToolResult } from './markdown'

describe('wrapCodeBlock', () => {
  it('uses 3 backticks when content has no fence lines', () => {
    const out = wrapCodeBlock('hello\nworld')
    expect(out).toBe('```\nhello\nworld\n```')
  })

  it('uses 4 backticks when content contains a bare 3-backtick line', () => {
    // A diff context line that is an unchanged code fence in the source file.
    const out = wrapCodeBlock('before\n ```\nafter')
    expect(out.startsWith('````\n')).toBe(true)
    expect(out.endsWith('\n````')).toBe(true)
  })

  it('scales the fence to exceed the longest all-backtick line', () => {
    const out = wrapCodeBlock('x\n````\ny') // 4-backtick line inside
    expect(out.startsWith('`````\n')).toBe(true) // 5-backtick wrapper
    expect(out.endsWith('\n`````')).toBe(true)
  })

  it('ignores backtick runs that share a line with other chars', () => {
    // `+```python` (a diff added line) is NOT a bare fence, so no extra fence
    // length is needed beyond the default 3.
    const out = wrapCodeBlock('+```python\n+print(1)')
    expect(out.startsWith('```\n')).toBe(true)
  })

  it('ignores all-backtick lines shorter than 3 (not valid fences)', () => {
    const out = wrapCodeBlock('a line with one: `\nand two: ``')
    expect(out.startsWith('```\n')).toBe(true)
  })
})

describe('formatToolResult', () => {
  it('wraps the result in a blockquote with a bold header', () => {
    const out = formatToolResult('get_branches', 'main\ndev')
    expect(out.startsWith('\n\n> **get_branches**')).toBe(true)
    expect(out).toContain('> ```')
    expect(out).toContain('> main')
    expect(out).toContain('> dev')
  })

  it('prefixes every line, including blanks, with "> "', () => {
    const out = formatToolResult('t', 'a\n\nb')
    // Drop the leading "\n\n" and the single trailing "\n" so split() does
    // not yield phantom empty elements at the boundaries.
    const body = out.slice(2, -1)
    for (const line of body.split('\n')) {
      expect(line.startsWith('> ')).toBe(true)
    }
  })

  // ----- Regression: the bug this helper exists to fix -----
  it('keeps content after an inner bare fence inside the code block', () => {
    // Tool result mimicking a diff of a Markdown file: the 3rd line is an
    // unchanged context line that is a code fence in the source (a single
    // leading space from the diff, then three backticks). With the old fixed
    // 3-backtick wrapper, marked closed the block at that line and parsed the
    // following "+new line" as a list item (leaked out of the code block).
    const result = ['File: x.md (+1 -0)', '@@ -1,2 +1,3 @@', ' ```', '+new line'].join('\n')
    const md = formatToolResult('get_branch_diff', result)
    const html = marked.parse(md, { gfm: true }) as string

    const codeOpen = html.indexOf('<code')
    const codeClose = html.indexOf('</code>')
    expect(codeOpen).toBeGreaterThan(-1)
    expect(codeClose).toBeGreaterThan(codeOpen)

    // The inner fence line must render as literal text inside the code block,
    // not as a fence that closes it early.
    const inner = html.slice(codeOpen, codeClose)
    expect(inner).toContain('```')
    // Crucially, content AFTER the inner fence must still be inside the code
    // block — this is exactly what broke before the fix.
    expect(inner).toContain('+new line')
  })
})
