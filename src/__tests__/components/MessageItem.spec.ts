import { mount } from '@vue/test-utils'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import MessageItem from '@/pages/chat/MessageItem.vue'
import ThinkingBlock from '@/pages/chat/ThinkingBlock.vue'
import type { Message, SearchResult } from '@/types'

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

    it('shows collapse button when more than 3 results', async () => {
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
      expect(wrapper.text()).toContain('结果1')
      expect(wrapper.text()).toContain('结果3')
      expect(wrapper.text()).not.toContain('结果4')
      expect(wrapper.text()).toContain('展开更多')
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

  describe('E2E: KIMI-style thinking + content + sources', () => {
    it('thinking mode: 默认折叠，显示标题，展开后可见内容', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({
            role: 'assistant',
            content: 'Rust 是一门系统编程语言[1]。',
            thinking_content: '用户问的是 Rust 语言的特点。让我分析一下...\n\n首先，Rust 的核心特性是内存安全。',
            search_results: [
              { title: 'Rust 官网', url: 'https://www.rust-lang.org' },
            ],
          }),
        },
      })

      // ThinkingBlock 应该存在
      const thinking = wrapper.findComponent(ThinkingBlock)
      expect(thinking.exists()).toBe(true)

      // 默认折叠：显示"思考了"标题，不显示思考内容
      expect(wrapper.text()).toContain('思考了')
      expect(wrapper.text()).not.toContain('核心特性是内存安全')

      // 展开后可以看到思考内容
      const toggleBtn = thinking.find('button')
      toggleBtn.trigger('click')
      return wrapper.vm.$nextTick().then(() => {
        expect(wrapper.text()).toContain('核心特性是内存安全')
      })
    })

    it('thinking 流式状态：显示"思考中..."动画', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({ role: 'assistant', content: '' }),
          streaming: true,
          streamingThinking: '正在分析用户的问题...',
          streamingThinkingStart: Date.now() - 3000,
        },
      })

      const thinking = wrapper.findComponent(ThinkingBlock)
      expect(thinking.exists()).toBe(true)
      expect(wrapper.text()).toContain('思考中...')
    })

    it('thinking 结束后消息正文完整渲染，顺序正确', () => {
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({
            role: 'assistant',
            content: 'Rust 是一门系统编程语言[1]，注重安全[2]。',
            thinking_content: '分析完成。',
            search_results: [
              { title: 'Rust 官网', url: 'https://www.rust-lang.org' },
              { title: 'Rust Book', url: 'https://doc.rust-lang.org/book' },
            ],
          }),
        },
      })

      const text = wrapper.text()

      // 1. Thinking 折叠标题存在
      expect(text).toContain('思考了')

      // 2. 消息正文存在
      expect(text).toContain('Rust 是一门系统编程语言')

      // 3. 搜索来源底栏在正文下方
      expect(text).toContain('2 个来源')

      // 验证顺序：thinking 在正文之前
      const thinkingIdx = text.indexOf('思考了')
      const contentIdx = text.indexOf('Rust 是一门系统编程语言')
      const sourcesIdx = text.indexOf('2 个来源')
      expect(thinkingIdx).toBeLessThan(contentIdx)
      expect(contentIdx).toBeLessThan(sourcesIdx)
    })

    it('消息正文下方显示搜索来源数目和 chip', () => {
      const searchResults: SearchResult[] = [
        { title: 'Rust 官网', url: 'https://www.rust-lang.org' },
        { title: 'Cargo 手册', url: 'https://doc.rust-lang.org/cargo' },
        { title: 'Crates.io', url: 'https://crates.io' },
      ]
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({
            role: 'assistant',
            content: '根据搜索结果回答...',
            search_results: searchResults,
          }),
        },
      })

      // 来源数目显示
      expect(wrapper.text()).toContain('3 个来源')

      // 来源 chip 显示（前3个）
      expect(wrapper.text()).toContain('Rust 官网')
      expect(wrapper.text()).toContain('Cargo 手册')
      expect(wrapper.text()).toContain('Crates.io')

      // 验证来源在正文之后
      const contentIdx = wrapper.text().indexOf('根据搜索结果回答')
      const sourcesIdx = wrapper.text().indexOf('3 个来源')
      expect(contentIdx).toBeLessThan(sourcesIdx)
    })

    it('流式 thinking + 搜索结果 + 正文的完整流程', () => {
      // 阶段1：thinking 流式输出中
      const wrapper = mount(MessageItem, {
        props: {
          message: createMsg({ role: 'assistant', content: '' }),
          streaming: true,
          streamingThinking: '让我搜索一下...',
          streamingThinkingStart: Date.now() - 5000,
          streamingSearchResults: [
            { title: 'Rust 官网', url: 'https://www.rust-lang.org' },
          ],
        },
      })

      // Thinking 折叠标题显示"思考中..."
      expect(wrapper.text()).toContain('思考中...')

      // ThinkingBlock 存在且 props 正确
      const thinking = wrapper.findComponent(ThinkingBlock)
      expect(thinking.exists()).toBe(true)
      expect(thinking.props('thinking')).toBe('让我搜索一下...')
      expect(thinking.props('streaming')).toBe(true)
      expect(thinking.props('searchResults')).toEqual([
        { title: 'Rust 官网', url: 'https://www.rust-lang.org' },
      ])

      // 展开后可以看到 thinking 内容
      const toggleBtn = thinking.find('button')
      toggleBtn.trigger('click')
      return wrapper.vm.$nextTick().then(() => {
        expect(wrapper.text()).toContain('让我搜索一下...')
      })
    })
  })
})
