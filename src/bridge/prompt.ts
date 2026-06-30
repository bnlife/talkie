import { invoke } from '@tauri-apps/api/core'
import type { Prompt } from '../types'

export async function listPrompts(): Promise<Prompt[]> {
  return invoke<Prompt[]>('list_prompts')
}

export async function createPrompt(name: string, content: string): Promise<Prompt> {
  return invoke<Prompt>('create_prompt', { name, content })
}

export async function updatePrompt(id: string, name: string, content: string): Promise<void> {
  return invoke<void>('update_prompt', { id, name, content })
}

export async function deletePrompt(id: string): Promise<void> {
  return invoke<void>('delete_prompt', { id })
}

export async function setDefaultPrompt(id: string): Promise<void> {
  return invoke<void>('set_default_prompt', { id })
}
