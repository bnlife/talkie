import { defineStore } from 'pinia'
import type { Prompt } from '../types'
import * as promptBridge from '../bridge/prompt'
import { log } from '../bridge/log'

export const usePromptStore = defineStore('prompt', {
  state: () => ({
    prompts: [] as Prompt[],
    activePromptId: null as string | null,
  }),

  getters: {
    activePrompt(state): Prompt | undefined {
      return state.prompts.find(p => p.id === state.activePromptId)
    },

    defaultPrompt(state): Prompt | undefined {
      return state.prompts.find(p => p.is_default)
    },
  },

  actions: {
    async loadPrompts(): Promise<void> {
      await log('info', '前端::promptStore::loadPrompts | 加载提示词列表')
      this.prompts = await promptBridge.listPrompts()
    },

    async createPrompt(name: string, content: string): Promise<Prompt> {
      await log('info', `前端::promptStore::createPrompt | 创建提示词 | name=${name}`)
      const prompt = await promptBridge.createPrompt(name, content)
      this.prompts.unshift(prompt)
      this.activePromptId = prompt.id
      return prompt
    },

    async updatePrompt(id: string, name: string, content: string): Promise<void> {
      await log('info', `前端::promptStore::updatePrompt | 更新提示词 | id=${id}`)
      await promptBridge.updatePrompt(id, name, content)
      const prompt = this.prompts.find(p => p.id === id)
      if (prompt) {
        prompt.name = name
        prompt.content = content
        prompt.updated_at = Date.now()
      }
    },

    async deletePrompt(id: string): Promise<void> {
      await log('info', `前端::promptStore::deletePrompt | 删除提示词 | id=${id}`)
      await promptBridge.deletePrompt(id)
      this.prompts = this.prompts.filter(p => p.id !== id)
      if (this.activePromptId === id) {
        this.activePromptId = null
      }
    },

    async setDefaultPrompt(id: string): Promise<void> {
      await log('info', `前端::promptStore::setDefaultPrompt | 设置默认提示词 | id=${id}`)
      await promptBridge.setDefaultPrompt(id)
      this.prompts.forEach(p => {
        p.is_default = p.id === id
      })
    },

    selectPrompt(id: string | null): void {
      this.activePromptId = id
    },
  },
})
