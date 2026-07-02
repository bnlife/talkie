import { describe, it, expect } from 'vitest'
import { readFileSync, readdirSync, statSync, existsSync } from 'fs'
import { join, extname, relative } from 'path'

const ROOT = join(import.meta.dirname, '..', '..')
const SCAN_DIRS = [
  join(ROOT, 'src', 'pages'),
  join(ROOT, 'src', 'components'),
]
const EXCLUDE_DIRS = ['ui']

interface Violation {
  file: string
  line: number
  content: string
  rule: string
}

function collectVueFiles(dir: string, files: string[] = []): string[] {
  if (!existsSync(dir)) return files
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry)
    const stat = statSync(full)
    if (stat.isDirectory()) {
      if (!EXCLUDE_DIRS.includes(entry)) {
        collectVueFiles(full, files)
      }
    } else if (entry.endsWith('.vue')) {
      files.push(full)
    }
  }
  return files
}

function extractTemplate(content: string): string {
  const start = content.indexOf('<template>')
  const end = content.lastIndexOf('</template>')
  if (start === -1 || end === -1) return ''
  return content.slice(start, end)
}

function getLineNumber(content: string, index: number): number {
  return content.slice(0, index).split('\n').length
}

const BUTTON_STYLE_OVERRIDES = [
  { pattern: /hover:/, rule: 'hover: on <Button> — use variant instead' },
  { pattern: /\bbg-(?!transparent\b)/, rule: 'bg- on <Button> — use variant instead' },
  { pattern: /\btext-(?!xs|sm|base|lg|xl|2xs|left|right|center\b)/, rule: 'text-{color} on <Button> — use variant instead' },
  { pattern: /\bshadow-/, rule: 'shadow- on <Button> — use variant instead' },
  { pattern: /\brounded-/, rule: 'rounded- on <Button> — use variant instead' },
  { pattern: /\bh-[0-9]/, rule: 'h-{n} on <Button> — use size prop instead' },
  { pattern: /\bsize-[0-9]/, rule: 'size-{n} on <Button> — use size prop instead' },
]

const INPUT_STYLE_OVERRIDES = [
  { pattern: /\bh-[0-9]/, rule: 'h-{n} on <Input> — use size prop instead' },
  { pattern: /\bbg-/, rule: 'bg- on <Input> — use size prop instead' },
  { pattern: /\bborder-(?!input\b)/, rule: 'border- on <Input> — use size prop instead' },
  { pattern: /\brounded-/, rule: 'rounded- on <Input> — use size prop instead' },
  { pattern: /\bshadow-/, rule: 'shadow- on <Input> — use size prop instead' },
  { pattern: /\bpx-[0-9]/, rule: 'px-{n} on <Input> — use size prop instead' },
  { pattern: /\bring-[0-9]/, rule: 'ring-{n} on <Input> — use size prop instead' },
  { pattern: /\bring-ring/, rule: 'ring-ring on <Input> — use size prop instead' },
]

function findClassAttr(template: string, tagStart: number, tag: string): { classStr: string; fullMatch: string; offset: number } | null {
  const tagEnd = template.indexOf('>', tagStart)
  if (tagEnd === -1) return null
  const tagContent = template.slice(tagStart, tagEnd)

  const classMatch = tagContent.match(/(?:class|:class)\s*=\s*"([^"]*)"/)
  if (!classMatch) return null

  return {
    classStr: classMatch[1],
    fullMatch: classMatch[0],
    offset: tagStart + tagContent.indexOf(classMatch[0]),
  }
}

function scanFile(filePath: string): Violation[] {
  const content = readFileSync(filePath, 'utf-8')
  const template = extractTemplate(content)
  if (!template) return []

  const violations: Violation[] = []
  const relFile = relative(ROOT, filePath)

  const buttonRegex = /<Button\b/g
  let m: RegExpExecArray | null
  while ((m = buttonRegex.exec(template)) !== null) {
    const classInfo = findClassAttr(template, m.index, 'Button')
    if (!classInfo) continue

    for (const { pattern, rule } of BUTTON_STYLE_OVERRIDES) {
      if (pattern.test(classInfo.classStr)) {
        violations.push({
          file: relFile,
          line: getLineNumber(template, classInfo.offset),
          content: classInfo.fullMatch,
          rule,
        })
      }
    }
  }

  const inputRegex = /<Input\b/g
  while ((m = inputRegex.exec(template)) !== null) {
    const classInfo = findClassAttr(template, m.index, 'Input')
    if (!classInfo) continue

    for (const { pattern, rule } of INPUT_STYLE_OVERRIDES) {
      if (pattern.test(classInfo.classStr)) {
        violations.push({
          file: relFile,
          line: getLineNumber(template, classInfo.offset),
          content: classInfo.fullMatch,
          rule,
        })
      }
    }
  }

  const nativeButtonRegex = /<button\b/g
  while ((m = nativeButtonRegex.exec(template)) !== null) {
    const tagEnd = template.indexOf('>', m.index)
    if (tagEnd === -1) continue
    const tagContent = template.slice(m.index, tagEnd)

    violations.push({
      file: relFile,
      line: getLineNumber(template, m.index),
      content: tagContent + '>',
      rule: 'native <button> — use <Button> component instead',
    })
  }

  const nativeInputRegex = /<input\b/g
  while ((m = nativeInputRegex.exec(template)) !== null) {
    const tagEnd = template.indexOf('>', m.index)
    if (tagEnd === -1) continue
    const tagContent = template.slice(m.index, tagEnd)

    if (/\btype\s*=\s*"file"/.test(tagContent)) continue

    violations.push({
      file: relFile,
      line: getLineNumber(template, m.index),
      content: tagContent + '>',
      rule: 'native <input> — use <Input> component instead',
    })
  }

  // 检测 DropdownMenu + Button 手动实现选择器（应用 Select 组件）
  // 特征：DropdownMenuTrigger 内有 Button，且 Button 内有 ChevronDown
  const dropdownMenuRegex = /<DropdownMenu\b/g
  while ((m = dropdownMenuRegex.exec(template)) !== null) {
    const menuEnd = template.indexOf('</DropdownMenu>', m.index)
    if (menuEnd === -1) continue
    const menuContent = template.slice(m.index, menuEnd)
    
    // 检查是否有 DropdownMenuTrigger + Button + ChevronDown 组合
    const hasTrigger = menuContent.includes('DropdownMenuTrigger')
    const hasButton = menuContent.includes('<Button')
    const hasChevron = menuContent.includes('ChevronDown')
    
    if (hasTrigger && hasButton && hasChevron) {
      violations.push({
        file: relFile,
        line: getLineNumber(template, m.index),
        content: '<DropdownMenu> with <Button> and <ChevronDown>',
        rule: 'DropdownMenu+Button pattern — use <Select> component instead',
      })
    }
  }

  return violations
}

describe('UI overrides lint', () => {
  it('no shadcn-vue component overrides or native elements in pages/components', () => {
    const files = SCAN_DIRS.flatMap(d => collectVueFiles(d))
    const allViolations: Violation[] = []

    for (const file of files) {
      allViolations.push(...scanFile(file))
    }

    if (allViolations.length > 0) {
      const report = allViolations.map(v =>
        `  ${v.file}:${v.line}  ${v.rule}\n    ${v.content.trim()}`,
      ).join('\n\n')
      expect.fail(
        `Found ${allViolations.length} UI override violation(s):\n\n${report}\n\n` +
        'Fix: modify the component source (src/components/ui/*) instead of overriding in usage.',
      )
    }
  })
})
