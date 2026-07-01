import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import MessageItem from '@/pages/chat/MessageItem.vue'
import type { Message } from '@/types'

function createMsg(overrides: Partial<Message> = {}): Message {
  return {
    id: 'msg-1',
    conversation_id: 'conv-1',
    role: 'user',
    content: 'hello',
    created_at: 100,
    ...overrides,
  }
}

describe('MessageItem.vue', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
  })

  it('renders message content', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ content: '测试消息' }) },
    })
    expect(wrapper.text()).toContain('测试消息')
  })

  it('renders user message with avatar', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ role: 'user' }) },
    })
    expect(wrapper.text()).toContain('你')
  })

  it('renders assistant message with avatar', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ role: 'assistant' }) },
    })
    expect(wrapper.text()).toContain('AI')
  })

  it('shows streaming indicator when streaming', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg(), streaming: true },
    })
    expect(wrapper.find('.animate-pulse').exists()).toBe(true)
  })

  it('shows copy and delete buttons', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg() },
    })
    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBe(2)
  })

  it('shows regenerate button for last assistant message', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ role: 'assistant' }), isLast: true },
    })
    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBe(3)
  })

  it('does not show regenerate button for non-last message', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ role: 'assistant' }), isLast: false },
    })
    const buttons = wrapper.findAll('button')
    expect(buttons.length).toBe(2)
  })

  it('copies content to clipboard on copy button click', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.assign(navigator, { clipboard: { writeText } })

    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ content: 'copy me' }) },
    })

    const copyBtn = wrapper.findAll('button')[0]
    await copyBtn.trigger('click')

    expect(writeText).toHaveBeenCalledWith('copy me')
  })

  it('changes icon to Check after copy', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.assign(navigator, { clipboard: { writeText } })

    const wrapper = mount(MessageItem, {
      props: { message: createMsg() },
    })

    const copyBtn = wrapper.findAll('button')[0]
    await copyBtn.trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.text-green-500').exists()).toBe(true)
  })

  it('emits delete event with message id', async () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ id: 'msg-to-delete' }) },
    })

    const deleteBtn = wrapper.findAll('button')[1]
    await deleteBtn.trigger('click')

    expect(wrapper.emitted('delete')).toBeTruthy()
    expect(wrapper.emitted('delete')![0]).toEqual(['msg-to-delete'])
  })

  it('emits regenerate event', async () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ role: 'assistant' }), isLast: true },
    })

    const regenerateBtn = wrapper.findAll('button')[2]
    await regenerateBtn.trigger('click')

    expect(wrapper.emitted('regenerate')).toBeTruthy()
  })

  it('does not show buttons when streaming', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg(), streaming: true },
    })
    expect(wrapper.findAll('button').length).toBe(0)
  })

  it('assistant message renders markdown (v-html)', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ role: 'assistant', content: '**加粗** 和 `代码`' }) },
    })
    const mdBody = wrapper.find('.markdown-body')
    expect(mdBody.exists()).toBe(true)
    expect(mdBody.html()).toContain('<strong>加粗</strong>')
    expect(mdBody.html()).toContain('<code')
    expect(mdBody.html()).toContain('代码')
  })

  it('user message uses plain text (not markdown)', () => {
    const wrapper = mount(MessageItem, {
      props: { message: createMsg({ role: 'user', content: '**不会加粗**' }) },
    })
    expect(wrapper.find('.markdown-body').exists()).toBe(false)
    expect(wrapper.text()).toContain('**不会加粗**')
  })

  describe('search results', () => {
    it('renders search results card when assistant message has search_results', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({
            role: 'assistant',
            content: '根据搜索结果...',
            search_results: [
              { title: 'Rust 官网', url: 'https://www.rust-lang.org', snippet: 'Rust 是一门系统编程语言' },
              { title: 'Cargo 手册', url: 'https://doc.rust-lang.org/cargo' },
            ],
          }),
        },
      })
      expect(wrapper.text()).toContain('2 个来源')
      expect(wrapper.text()).toContain('Rust 官网')
      expect(wrapper.text()).toContain('Cargo 手册')
    })

    it('does not render search results card when search_results is empty', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({ role: 'assistant', content: '你好', search_results: [] }),
        },
      })
      expect(wrapper.text()).not.toContain('个来源')
    })

    it('does not render search results card when search_results is undefined', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({ role: 'assistant', content: '你好' }),
        },
      })
      expect(wrapper.text()).not.toContain('个来源')
    })

    it('does not render search results for user messages', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({
            role: 'user',
            content: '搜索 Rust',
            search_results: [{ title: 'Rust', url: 'https://rust-lang.org' }],
          }),
        },
      })
      expect(wrapper.text()).not.toContain('个来源')
    })

    it('shows collapse button when more than 5 results', async () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({
            role: 'assistant',
            content: '根据搜索结果...',
            search_results: [
              { title: '结果1', url: 'https://a.com' },
              { title: '结果2', url: 'https://b.com' },
              { title: '结果3', url: 'https://c.com' },
              { title: '结果4', url: 'https://d.com' },
              { title: '结果5', url: 'https://e.com' },
              { title: '结果6', url: 'https://f.com' },
            ],
          }),
        },
      })
      // Shows first 5 and collapse button
      expect(wrapper.text()).toContain('结果1')
      expect(wrapper.text()).toContain('结果5')
      expect(wrapper.text()).not.toContain('结果6')
      expect(wrapper.text()).toContain('+1')
    })

    it('renders source chip with title', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({
            role: 'assistant',
            content: '根据搜索结果...',
            search_results: [
              { title: 'Rust 官网', url: 'https://www.rust-lang.org', snippet: 'Rust 是一门系统编程语言' },
            ],
          }),
        },
      })
      expect(wrapper.text()).toContain('Rust 官网')
    })

    it('shows source title in chip', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({
            role: 'assistant',
            content: '根据搜索结果...',
            search_results: [
              { title: 'Rust 官网', url: 'https://www.rust-lang.org/learn' },
            ],
          }),
        },
      })
      expect(wrapper.text()).toContain('Rust 官网')
    })
  })
})
