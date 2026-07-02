import { computed, toRef, type Ref } from 'vue'
import type { SearchResult } from '@/types'
import { renderMarkdown } from '@/lib/markdown'

export function useMessageRender(
  content: Ref<string>,
  searchResults: Ref<SearchResult[]>,
  isUser: Ref<boolean>,
) {
  const urlIndexMap = computed(() => {
    const map = new Map<string, number>()
    searchResults.value.forEach((r, i) => {
      if (r.url) map.set(r.url, i + 1)
    })
    return map
  })

  const renderedHtml = computed(() => {
    if (isUser.value) return ''
    let html = renderMarkdown(content.value)
    if (searchResults.value.length === 0) return html

    html = html.replace(
      /<a\s+[^>]*href="([^"]*)"[^>]*>(.*?)<\/a>/gi,
      (match, href) => {
        const idx = urlIndexMap.value.get(href)
        if (idx) {
          const url = searchResults.value[idx - 1]?.url ?? href
          return `${match}<sup class="search-citation" data-url="${url}">${idx}</sup>`
        }
        return match
      }
    )

    html = html.replace(
      /(?<!<[^>]*?)\[(\d+)\](?![^<]*?<\/(?:code|pre)>)/g,
      (match, num) => {
        const idx = parseInt(num, 10)
        if (idx >= 1 && idx <= searchResults.value.length) {
          const url = searchResults.value[idx - 1]?.url ?? ''
          return `<sup class="search-citation" data-url="${url}">${idx}</sup>`
        }
        return match
      }
    )

    return html
  })

  return {
    urlIndexMap,
    renderedHtml,
  }
}

export function getDomain(url: string): string {
  try { return new URL(url).hostname.replace(/^www\./, '') } catch { return url }
}

export function getFaviconUrl(url: string): string {
  try { return `https://www.google.com/s2/favicons?domain=${new URL(url).hostname}&sz=16` } catch { return '' }
}