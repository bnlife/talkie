import { describe, it, expect } from 'vitest'
import {
  isTextFile,
  validateFileSize,
  getLanguageHint,
  readFileAsText,
  formatAttachments,
} from '../../lib/attachment'

function makeFile(name: string, size: number, type = ''): File {
  const content = 'x'.repeat(size)
  return new File([content], name, { type })
}

describe('isTextFile', () => {
  it('识别常见文本扩展名', () => {
    expect(isTextFile(makeFile('main.js', 10))).toBe(true)
    expect(isTextFile(makeFile('app.ts', 10))).toBe(true)
    expect(isTextFile(makeFile('index.tsx', 10))).toBe(true)
    expect(isTextFile(makeFile('script.py', 10))).toBe(true)
    expect(isTextFile(makeFile('lib.rs', 10))).toBe(true)
    expect(isTextFile(makeFile('main.go', 10))).toBe(true)
    expect(isTextFile(makeFile('App.java', 10))).toBe(true)
    expect(isTextFile(makeFile('main.c', 10))).toBe(true)
    expect(isTextFile(makeFile('lib.cpp', 10))).toBe(true)
    expect(isTextFile(makeFile('config.json', 10))).toBe(true)
    expect(isTextFile(makeFile('data.yaml', 10))).toBe(true)
    expect(isTextFile(makeFile('style.css', 10))).toBe(true)
    expect(isTextFile(makeFile('index.html', 10))).toBe(true)
    expect(isTextFile(makeFile('readme.md', 10))).toBe(true)
    expect(isTextFile(makeFile('notes.txt', 10))).toBe(true)
    expect(isTextFile(makeFile('query.sql', 10))).toBe(true)
    expect(isTextFile(makeFile('run.sh', 10))).toBe(true)
    expect(isTextFile(makeFile('Component.vue', 10))).toBe(true)
    expect(isTextFile(makeFile('config.toml', 10))).toBe(true)
    expect(isTextFile(makeFile('Dockerfile', 10, 'text/plain'))).toBe(true)
  })

  it('识别 application/json 等 MIME 类型', () => {
    expect(isTextFile(makeFile('data', 10, 'application/json'))).toBe(true)
    expect(isTextFile(makeFile('data', 10, 'application/javascript'))).toBe(true)
    expect(isTextFile(makeFile('data', 10, 'application/typescript'))).toBe(true)
    expect(isTextFile(makeFile('data', 10, 'application/xml'))).toBe(true)
    expect(isTextFile(makeFile('data', 10, 'application/sql'))).toBe(true)
    expect(isTextFile(makeFile('data', 10, 'application/x-sh'))).toBe(true)
  })

  it('识别 text/* MIME 类型', () => {
    expect(isTextFile(makeFile('data', 10, 'text/plain'))).toBe(true)
    expect(isTextFile(makeFile('data', 10, 'text/html'))).toBe(true)
    expect(isTextFile(makeFile('data', 10, 'text/css'))).toBe(true)
    expect(isTextFile(makeFile('data', 10, 'text/javascript'))).toBe(true)
  })

  it('扩展名优先于 MIME 类型', () => {
    expect(isTextFile(makeFile('main.rs', 10, 'application/octet-stream'))).toBe(true)
    expect(isTextFile(makeFile('data.json', 10, 'application/octet-stream'))).toBe(true)
  })

  it('拒绝非文本文件', () => {
    expect(isTextFile(makeFile('photo.png', 10, 'image/png'))).toBe(false)
    expect(isTextFile(makeFile('photo.jpg', 10, 'image/jpeg'))).toBe(false)
    expect(isTextFile(makeFile('video.mp4', 10, 'video/mp4'))).toBe(false)
    expect(isTextFile(makeFile('audio.mp3', 10, 'audio/mpeg'))).toBe(false)
    expect(isTextFile(makeFile('doc.pdf', 10, 'application/pdf'))).toBe(false)
    expect(isTextFile(makeFile('archive.zip', 10, 'application/zip'))).toBe(false)
  })
})

describe('validateFileSize', () => {
  it('500KB 以内的文件通过', () => {
    expect(validateFileSize(makeFile('a.txt', 500 * 1024)).ok).toBe(true)
  })

  it('超过 500KB 被拒绝', () => {
    const result = validateFileSize(makeFile('big.txt', 500 * 1024 + 1))
    expect(result.ok).toBe(false)
    expect(result.error).toContain('big.txt')
    expect(result.error).toContain('500KB')
  })

  it('空文件通过', () => {
    expect(validateFileSize(makeFile('empty.txt', 0)).ok).toBe(true)
  })
})

describe('getLanguageHint', () => {
  it('返回正确的语言标识', () => {
    expect(getLanguageHint('main.rs')).toBe('rust')
    expect(getLanguageHint('app.py')).toBe('python')
    expect(getLanguageHint('index.ts')).toBe('typescript')
    expect(getLanguageHint('script.js')).toBe('javascript')
    expect(getLanguageHint('main.go')).toBe('go')
    expect(getLanguageHint('App.java')).toBe('java')
    expect(getLanguageHint('main.c')).toBe('c')
    expect(getLanguageHint('lib.cpp')).toBe('cpp')
    expect(getLanguageHint('style.css')).toBe('css')
    expect(getLanguageHint('data.json')).toBe('json')
    expect(getLanguageHint('config.yaml')).toBe('yaml')
    expect(getLanguageHint('query.sql')).toBe('sql')
    expect(getLanguageHint('run.sh')).toBe('bash')
    expect(getLanguageHint('readme.md')).toBe('markdown')
    expect(getLanguageHint('App.vue')).toBe('vue')
  })

  it('未知扩展名返回空字符串', () => {
    expect(getLanguageHint('data.xyz')).toBe('')
    expect(getLanguageHint('noext')).toBe('')
    expect(getLanguageHint('.gitignore')).toBe('')
  })
})

describe('readFileAsText', () => {
  it('读取文件内容为字符串', async () => {
    const file = new File(['hello world'], 'test.txt', { type: 'text/plain' })
    const content = await readFileAsText(file)
    expect(content).toBe('hello world')
  })

  it('读取空文件返回空字符串', async () => {
    const file = new File([''], 'empty.txt', { type: 'text/plain' })
    const content = await readFileAsText(file)
    expect(content).toBe('')
  })

  it('读取包含中文的内容', async () => {
    const file = new File(['你好世界'], 'cn.txt', { type: 'text/plain' })
    const content = await readFileAsText(file)
    expect(content).toBe('你好世界')
  })
})

describe('formatAttachments', () => {
  it('无附件时返回原始文字', () => {
    expect(formatAttachments('hello', [])).toBe('hello')
  })

  it('单个附件正确拼接', () => {
    const result = formatAttachments('分析代码', [{ name: 'main.rs', content: 'fn main() {}' }])
    expect(result).toContain('分析代码')
    expect(result).toContain('### 📎 附件: main.rs')
    expect(result).toContain('```rust')
    expect(result).toContain('fn main() {}')
  })

  it('多个附件按顺序拼接', () => {
    const result = formatAttachments('看下这两个', [
      { name: 'a.py', content: 'print("a")' },
      { name: 'b.js', content: 'console.log("b")' },
    ])
    const idxA = result.indexOf('a.py')
    const idxB = result.indexOf('b.js')
    expect(idxA).toBeGreaterThan(0)
    expect(idxB).toBeGreaterThan(idxA)
    expect(result).toContain('```python')
    expect(result).toContain('```javascript')
  })

  it('附件内容含反引号不破坏格式', () => {
    const result = formatAttachments('test', [{
      name: 'code.md',
      content: '这里有 `代码` 和 ```三反引号```',
    }])
    expect(result).toContain('📎 附件: code.md')
    expect(result).toContain('`代码`')
  })

  it('空文件仍然显示附件标题', () => {
    const result = formatAttachments('test', [{ name: 'empty.txt', content: '' }])
    expect(result).toContain('📎 附件: empty.txt')
    expect(result).toContain('```')
  })

  it('用户文字为空时只显示附件', () => {
    const result = formatAttachments('', [{ name: 'a.txt', content: 'data' }])
    expect(result).not.toContain('分析')
    expect(result).toContain('📎 附件: a.txt')
  })

  it('用户文字为纯空白时只显示附件', () => {
    const result = formatAttachments('   \n  ', [{ name: 'a.txt', content: 'data' }])
    expect(result).toContain('📎 附件: a.txt')
    const firstLine = result.split('\n')[0]
    expect(firstLine).toContain('---')
  })

  it('附件大小显示正确', () => {
    const content = 'a'.repeat(1024)
    const result = formatAttachments('test', [{ name: 'big.txt', content }])
    expect(result).toContain('1.0 KB')
  })
})
