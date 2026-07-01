import { describe, it, expect } from 'vitest'
import { renderMarkdown } from '../../lib/markdown'

describe('renderMarkdown', () => {
  it('基础格式: 标题/加粗/斜体/列表', () => {
    const input = '# 标题\n\n**加粗** 和 *斜体*\n\n- 列表项1\n- 列表项2'
    const html = renderMarkdown(input)
    expect(html).toContain('<h1')
    expect(html).toContain('标题')
    expect(html).toContain('<strong>加粗</strong>')
    expect(html).toContain('<em>斜体</em>')
    expect(html).toContain('<li>')
    expect(html).toContain('列表项1')
  })

  it('代码块: pre+code 正确生成', () => {
    const input = '```js\nconsole.log("hello")\n```'
    const html = renderMarkdown(input)
    expect(html).toContain('<pre>')
    expect(html).toContain('<code')
    expect(html).toContain('console.log')
  })

  it('行内代码', () => {
    const input = '使用 `npm install` 安装'
    const html = renderMarkdown(input)
    expect(html).toContain('<code')
    expect(html).toContain('npm install')
  })

  it('表格', () => {
    const input = '| A | B |\n|---|---|\n| 1 | 2 |'
    const html = renderMarkdown(input)
    expect(html).toContain('<table>')
    expect(html).toContain('<th')
    expect(html).toContain('<td')
    expect(html).toContain('1')
    expect(html).toContain('2')
  })

  it('引用块', () => {
    const input = '> 这是一段引用'
    const html = renderMarkdown(input)
    expect(html).toContain('<blockquote>')
    expect(html).toContain('这是一段引用')
  })

  it('链接', () => {
    const input = '[点击这里](https://example.com)'
    const html = renderMarkdown(input)
    expect(html).toContain('<a')
    expect(html).toContain('href="https://example.com"')
    expect(html).toContain('点击这里')
  })

  it('XSS 过滤: script 标签被移除', () => {
    const input = '<script>alert("xss")</script>正常文本'
    const html = renderMarkdown(input)
    expect(html).not.toContain('<script>')
    expect(html).toContain('正常文本')
  })

  it('XSS 过滤: onerror 属性被移除', () => {
    const input = '<img src=x onerror=alert(1)>'
    const html = renderMarkdown(input)
    expect(html).not.toContain('onerror')
  })

  it('空文本返回空字符串', () => {
    expect(renderMarkdown('')).toBe('')
  })

  it('流式中间态: 未闭合的代码块不会崩溃', () => {
    const input = '一些文本\n```js\nconsole.log("未闭合")'
    expect(() => renderMarkdown(input)).not.toThrow()
    const html = renderMarkdown(input)
    expect(html).toContain('console.log')
  })

  it('删除线', () => {
    const input = '~~删除的文字~~'
    const html = renderMarkdown(input)
    expect(html).toContain('<del>')
    expect(html).toContain('删除的文字')
  })

  it('有序列表', () => {
    const input = '1. 第一项\n2. 第二项'
    const html = renderMarkdown(input)
    expect(html).toContain('<ol>')
    expect(html).toContain('<li>')
    expect(html).toContain('第一项')
  })

  it('换行符转换 (breaks: true)', () => {
    const input = '第一行\n第二行'
    const html = renderMarkdown(input)
    expect(html).toContain('<br')
  })
})
