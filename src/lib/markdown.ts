import { marked } from 'marked'
import DOMPurify from 'dompurify'

const renderer = new marked.Renderer()
renderer.link = function ({ href, text }) {
  return `<a href="${href}" target="_blank" rel="noopener">${text}</a>`
}

marked.setOptions({
  breaks: true,
  gfm: true,
  renderer,
})

const ALLOWED_TAGS = [
  'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'p', 'br', 'hr',
  'strong', 'em', 'del', 'code', 'pre',
  'ul', 'ol', 'li',
  'blockquote',
  'table', 'thead', 'tbody', 'tr', 'th', 'td',
  'a',
]

const ALLOWED_ATTR = ['href', 'title', 'target', 'rel']

export function renderMarkdown(text: string): string {
  if (!text) return ''
  const rawHtml = marked.parse(text) as string
  return DOMPurify.sanitize(rawHtml, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
  })
}
