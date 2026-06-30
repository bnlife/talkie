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
})
