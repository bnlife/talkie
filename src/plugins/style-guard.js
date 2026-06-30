const VIOLATION_PATTERNS = [
  // Tailwind 任意值
  /text-\[[\s\S]*?\]/,
  /w-\[[\s\S]*?\]/,
  /h-\[[\s\S]*?\]/,
  /bg-\[[\s\S]*?\]/,
  /rounded-\[[\s\S]*?\]/,
  /p-\[[\s\S]*?\]/,
  /m-\[[\s\S]*?\]/,
  /gap-\[[\s\S]*?\]/,

  // 原生 Tailwind 字号类
  /\btext-xs\b/, /\btext-sm\b/, /\btext-base\b/, /\btext-lg\b/,
  /\btext-xl\b/, /\btext-2xl\b/, /\btext-3xl\b/, /\btext-4xl\b/,
  /\btext-5xl\b/, /\btext-6xl\b/, /\btext-7xl\b/, /\btext-8xl\b/, /\btext-9xl\b/,

  // 原生 Tailwind 颜色类
  /\b(slate|gray|zinc|neutral|stone|red|orange|amber|yellow|lime|green|emerald|teal|cyan|sky|blue|indigo|violet|purple|fuchsia|pink|rose)-[0-9]+\b/,

  // 原生 Tailwind 间距类
  /\bp-(0|px|0\.5|1|1\.5|2|2\.5|3|3\.5|4|5|6|7|8|9|10|11|12|14|16|20|24|28|32|36|40|44|48|52|56|60|64|72|80|96)\b/,

  // 原生 Tailwind 圆角类
  /\brounded-none\b/, /\brounded-sm\b/, /\brounded\b(?!-)/,
  /\brounded-md\b/, /\brounded-lg\b/, /\brounded-xl\b/,
  /\brounded-2xl\b/, /\brounded-3xl\b/, /\brounded-full\b/,

  // 硬编码单位
  /(?<![a-zA-Z-])[0-9]+px(?![a-zA-Z])/, /(?<![a-zA-Z-])[0-9.]+rem(?![a-zA-Z])/, /(?<![a-zA-Z-])[0-9.]+em(?![a-zA-Z])/, /(?<![a-zA-Z-])[0-9.]+v[w,h](?![a-zA-Z])/, /(?<![a-zA-Z-])[0-9]+%/,

  // !important
  /!important/,

  // <style> 标签
  /<style[^>]*>/,

  // 内联样式
  /style="[^"]*"/, /style='[^']*'/,

  // hex 颜色
  /#[0-9a-fA-F]{3,8}\b/,

  // rgb/rgba/hsl/hsla
  /\brgba?\s*\(/, /\bhsla?\s*\(/,
]

export default function styleGuard() {
  return {
    name: 'vite-plugin-style-guard',
    enforce: 'pre',
    transform(code, id) {
      if (!id.includes('/src/')) return
      if (id.includes('/node_modules/')) return
      if (id.includes('/components/ui/')) return
      if (id.endsWith('.spec.ts')) return
      if (!id.endsWith('.vue') && !id.endsWith('.ts') && !id.endsWith('.tsx')) return

      const lines = code.split('\n')
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i]
        for (const pattern of VIOLATION_PATTERNS) {
          if (pattern.test(line)) {
            throw new Error(
              `[STYLE GUARD] 违规: "${pattern.source}" 在文件 ${id} 第 ${i + 1} 行\n` +
              `  >>> ${line.trim()}`
            )
          }
        }
      }
    },
  }
}
