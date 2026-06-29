import { invoke } from '@tauri-apps/api/core'

export async function log(level: string, message: string) {
  try {
    await invoke('log_message', { level, message })
  } catch {
    // 日志写入失败不做处理，避免死循环
  }
}
