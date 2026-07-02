export interface AttachmentMeta {
  name: string
  size: number
}

export interface PendingAttachment {
  file: File
  name: string
  size: number
}

const MAX_FILE_SIZE = 500 * 1024 // 500KB

const TEXT_MIME_PREFIXES = ['text/']

const TEXT_MIME_TYPES = new Set([
  'application/json',
  'application/javascript',
  'application/typescript',
  'application/xml',
  'application/x-sh',
  'application/x-javascript',
  'application/x-typescript',
  'application/sql',
  'application/toml',
  'application/x-yaml',
  'application/yaml',
  'application/x-httpd-php',
])

const TEXT_EXTENSIONS = new Set([
  '.txt', '.md', '.markdown', '.json', '.jsonl', '.js', '.mjs', '.cjs',
  '.ts', '.tsx', '.jsx', '.vue', '.svelte', '.astro',
  '.py', '.rs', '.go', '.java', '.c', '.cpp', '.cc', '.cxx', '.h', '.hpp',
  '.cs', '.swift', '.kt', '.scala', '.lua', '.r', '.rb', '.php',
  '.css', '.scss', '.less', '.html', '.htm', '.xml', '.svg',
  '.yaml', '.yml', '.toml', '.ini', '.cfg', '.conf',
  '.sql', '.sh', '.bash', '.zsh', '.fish', '.bat', '.cmd', '.ps1',
  '.dockerfile', '.makefile', '.cmake', '.gradle',
  '.env', '.gitignore', '.editorconfig', '.prettierrc', '.eslintrc',
  '.config',
])

const LANG_MAP: Record<string, string> = {
  '.js': 'javascript', '.mjs': 'javascript', '.cjs': 'javascript',
  '.ts': 'typescript', '.tsx': 'typescript', '.jsx': 'javascript',
  '.py': 'python', '.rs': 'rust', '.go': 'go', '.java': 'java',
  '.c': 'c', '.cpp': 'cpp', '.cc': 'cpp', '.cxx': 'cpp', '.h': 'c', '.hpp': 'cpp',
  '.cs': 'csharp', '.swift': 'swift', '.kt': 'kotlin', '.scala': 'scala',
  '.lua': 'lua', '.r': 'r', '.rb': 'ruby', '.php': 'php',
  '.css': 'css', '.scss': 'scss', '.less': 'less',
  '.html': 'html', '.htm': 'html', '.xml': 'xml', '.svg': 'xml',
  '.json': 'json', '.yaml': 'yaml', '.yml': 'yaml', '.toml': 'toml',
  '.sql': 'sql', '.sh': 'bash', '.bash': 'bash', '.zsh': 'zsh',
  '.bat': 'batch', '.cmd': 'batch', '.ps1': 'powershell',
  '.md': 'markdown', '.markdown': 'markdown',
  '.vue': 'vue', '.svelte': 'svelte',
  '.dockerfile': 'dockerfile', '.makefile': 'makefile',
}

function getExtension(filename: string): string {
  const dot = filename.lastIndexOf('.')
  return dot >= 0 ? filename.slice(dot).toLowerCase() : ''
}

export function isTextFile(file: File): boolean {
  const ext = getExtension(file.name)
  if (TEXT_EXTENSIONS.has(ext)) return true

  const mime = (file.type || '').toLowerCase()
  if (TEXT_MIME_PREFIXES.some(p => mime.startsWith(p))) return true
  if (TEXT_MIME_TYPES.has(mime)) return true

  return false
}

export function validateFileSize(file: File): { ok: boolean; error?: string } {
  if (file.size > MAX_FILE_SIZE) {
    const kb = Math.round(file.size / 1024)
    return { ok: false, error: `文件 ${file.name} 太大 (${kb}KB)，上限 500KB` }
  }
  return { ok: true }
}

export function getLanguageHint(filename: string): string {
  const ext = getExtension(filename)
  return LANG_MAP[ext] || ''
}

export function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = () => reject(new Error(`读取文件 ${file.name} 失败`))
    reader.readAsText(file)
  })
}

export function formatAttachments(
  text: string,
  attachments: { name: string; content: string }[],
): string {
  if (attachments.length === 0) return text

  const parts: string[] = []
  if (text.trim()) parts.push(text.trim())

  for (const att of attachments) {
    const lang = getLanguageHint(att.name)
    const sizeKB = (new TextEncoder().encode(att.content).length / 1024).toFixed(1)
    const fence = '```'
    parts.push(
      `---\n### \uD83D\uDCCE \u9644\u4EF6: ${att.name} (${sizeKB} KB)\n${fence}${lang}\n${att.content}\n${fence}`,
    )
  }

  return parts.join('\n\n')
}
