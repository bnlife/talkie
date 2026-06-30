import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './styles/tokens.css'
import App from './App.vue'
import { log } from './bridge/log'

const app = createApp(App)

// Vue 错误兜底
app.config.errorHandler = (err, _instance, _info) => {
  log('error', `ERR_internal | Vue 错误: ${err}`)
}

app.use(createPinia())
app.mount('#app')

// 未捕获 Promise 拒绝
window.addEventListener('unhandledrejection', (event) => {
  log('error', `ERR_internal | 未捕获 Promise 拒绝: ${event.reason}`)
})

// 运行时异常
window.addEventListener('error', (event) => {
  log('error', `ERR_internal | 运行时异常: ${event.message}`)
})
