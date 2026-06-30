// vitest 环境配置
//  mock Tauri API，防止组件测试中报 "invoke is not defined"

import { vi } from 'vitest'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock Tauri event listen
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}))

// ResizeObserver polyfill
globalThis.ResizeObserver = class ResizeObserver { observe() {} unobserve() {} disconnect() {} }
