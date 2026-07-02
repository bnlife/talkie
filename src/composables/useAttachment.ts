import { ref, computed } from 'vue'
import { toast } from 'vue-sonner'
import { log } from '@/bridge/log'
import { isTextFile, validateFileSize, readFileAsText, formatAttachments } from '@/lib/attachment'
import type { PendingAttachment, AttachmentMeta } from '@/lib/attachment'

export function useAttachment() {
  const attachments = ref<PendingAttachment[]>([])
  const fileInputRef = ref<HTMLInputElement | null>(null)
  const justAdded = ref<Set<string>>(new Set())

  function markJustAdded(key: string) {
    justAdded.value.add(key)
    setTimeout(() => {
      justAdded.value.delete(key)
    }, 800)
  }

  function addFiles(files: FileList | File[]) {
    let rejected = 0
    let duplicate = 0

    for (const file of Array.from(files)) {
      if (!isTextFile(file)) {
        toast.warning(`${file.name} 不是文本文件，已跳过`)
        rejected++
        continue
      }
      const sizeCheck = validateFileSize(file)
      if (!sizeCheck.ok) {
        toast.warning(sizeCheck.error!)
        rejected++
        continue
      }
      const key = `${file.name}-${file.size}`
      if (attachments.value.some(a => `${a.name}-${a.size}` === key)) {
        duplicate++
        continue
      }
      attachments.value.push({ file, name: file.name, size: file.size })
      markJustAdded(key)
    }

    const added = Array.from(files).length - rejected - duplicate
    if (added > 0) {
      log('info', `FE::useAttachment | add | added=${added} rejected=${rejected} duplicate=${duplicate}`)
    }
  }

  function removeAttachment(index: number) {
    const removed = attachments.value[index]
    attachments.value.splice(index, 1)
    log('debug', `FE::useAttachment | remove | name=${removed?.name}`)
  }

  function triggerFileInput() {
    fileInputRef.value?.click()
  }

  function handleFileChange(e: Event) {
    const target = e.target as HTMLInputElement
    if (target.files) {
      log('debug', `FE::useAttachment | select | source=input count=${target.files.length}`)
      addFiles(target.files)
    }
    target.value = ''
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault()
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault()
    if (e.dataTransfer?.files) {
      log('debug', `FE::useAttachment | select | source=drop count=${e.dataTransfer.files.length}`)
      addFiles(e.dataTransfer.files)
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`
    return `${(bytes / 1024).toFixed(1)} KB`
  }

  const canSend = computed(() => attachments.value.length > 0)

  async function buildContent(text: string): Promise<{ displayContent: string; fullContent: string; metas?: AttachmentMeta[] }> {
    const pending = [...attachments.value]
    if (pending.length === 0) {
      return { displayContent: text, fullContent: text }
    }

    const resolved = await Promise.all(
      pending.map(async (att) => {
        try {
          return { name: att.name, content: await readFileAsText(att.file) }
        } catch (e) {
          log('error', `FE::useAttachment | read fail | name=${att.name} err=${e}`)
          toast.error(`读取 ${att.name} 失败`)
          return { name: att.name, content: '' }
        }
      }),
    )

    const metas: AttachmentMeta[] = pending.map((a, i) => ({ name: a.name, size: a.size, content: resolved[i].content }))
    const fullContent = formatAttachments(text, resolved)

    log('info', `FE::useAttachment | build | files=${pending.length} totalLen=${fullContent.length}`)
    attachments.value = []

    return { displayContent: text, fullContent, metas }
  }

  function clearAttachments() {
    attachments.value = []
  }

  return {
    attachments,
    fileInputRef,
    justAdded,
    addFiles,
    removeAttachment,
    triggerFileInput,
    handleFileChange,
    handleDragOver,
    handleDrop,
    formatSize,
    canSend,
    buildContent,
    clearAttachments,
  }
}
