<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue"
import { useRoute } from "vue-router"
import type { CharacterExpression, CharacterModel, CharacterLlmConfig } from "@/types"
import { listExpressions } from "@/api/expression"
import { ensureDefaultExpressions, getLlmConfig } from "@/api/llm"
import { openPetDevtools } from "@/api/pet-window"
import { getModel } from "@/api/model"
import { callLlm, buildCharacterPrompt, type LlmResponse, type ChatMessage } from "@/api/chat"
import { invoke } from "@tauri-apps/api/core"

const route = useRoute()
const modelId = computed(() => Number(route.params.id))

const exprMap = ref<Map<string, CharacterExpression>>(new Map())
const contextMenu = ref(false)
const alwaysOnTop = ref(true)
const imgError = ref(false)
const loadError = ref("")
const pageReady = ref(false)     // 标记 Vue 渲染完成
const initialized = ref(false)   // 标记数据加载完成
const dataUrl = ref("")          // 图片 base64 data URL
const audioRef = ref<HTMLAudioElement | null>(null)

// 底部操作栏
const modelInfo = ref<CharacterModel | null>(null)
const chatText = ref("")
const ttsText = ref("你好，我是你的桌宠！")
const isSpeaking = ref(false)

// LLM 聊天
const llmConfig = ref<CharacterLlmConfig | null>(null)
const chatHistory = ref<ChatMessage[]>([])
const isThinking = ref(false)

// 用动态 import 避免在非 Tauri 环境报错
let appWindow: any = null
let appWebviewWindow: any = null

async function initWindow() {
  try {
    // 尝试两种 API 获取当前窗口
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window")
      appWindow = getCurrentWindow()
    } catch { /* ignore */ }
    try {
      const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow")
      appWebviewWindow = getCurrentWebviewWindow()
    } catch { /* ignore */ }
    console.log("[PetWindow] appWindow:", !!appWindow, "appWebviewWindow:", !!appWebviewWindow)
  } catch {
    console.warn("[PetWindow] 非 Tauri 环境或 API 加载失败")
  }
}

const currentExprName = ref("default")
const currentExpr = computed(() => exprMap.value.get(currentExprName.value))
const illustrationPath = computed(() => currentExpr.value?.illustration_path || "")

// 当立绘路径变化时，通过 IPC 读取本地文件为 data URL
watch(illustrationPath, async (path) => {
  if (path) {
    try {
      dataUrl.value = await invoke<string>("read_image_as_data_url", { path })
      imgError.value = false
    } catch (e) {
      console.error("[PetWindow] 读取立绘失败:", e)
      imgError.value = true
    }
  } else {
    dataUrl.value = ""
    imgError.value = false
  }
})

async function loadExpressions() {
  try {
    console.log("[PetWindow] 开始加载表情数据, modelId:", modelId.value)
    // 确保默认表情存在
    await ensureDefaultExpressions(modelId.value)
    console.log("[PetWindow] ensureDefaultExpressions 完成")

    const exprs = await listExpressions(modelId.value)
    console.log("[PetWindow] listExpressions 完成, 数量:", exprs.length)

    const map = new Map<string, CharacterExpression>()
    for (const e of exprs) {
      map.set(e.name, e)
    }
    exprMap.value = map
    if (map.size === 0) {
      loadError.value = "该角色暂无表情数据"
    }
    initialized.value = true
  } catch (e: any) {
    loadError.value = `加载表情失败: ${e}`
    console.error("[PetWindow] loadExpressions error:", e)
    initialized.value = true
  }
}

async function loadModelInfo() {
  try {
    const m = await getModel(modelId.value)
    modelInfo.value = m
    console.log("[PetWindow] 模型信息加载完成:", m?.name)
  } catch (e) {
    console.error("[PetWindow] 加载模型信息失败:", e)
  }
}

async function loadLlmConfig() {
  try {
    const cfg = await getLlmConfig(modelId.value)
    llmConfig.value = cfg
    if (cfg?.greeting_message) {
      chatText.value = cfg.greeting_message
    }
    console.log("[PetWindow] LLM 配置加载完成:", cfg ? "有配置" : "无配置")
  } catch (e) {
    console.error("[PetWindow] 加载 LLM 配置失败:", e)
  }
}

/** 切换到指定表情 */
function switchExpression(name: string) {
  if (exprMap.value.has(name)) {
    currentExprName.value = name
  } else {
    currentExprName.value = "default"
  }
}

/** 发送聊天消息 → LLM → 切换表情 + 语音回复 */
async function sendChat() {
  const text = chatText.value.trim()
  if (!text || isThinking.value) return

  const cfg = llmConfig.value
  if (!cfg?.system_prompt) {
    console.warn("[PetWindow] LLM 未配置 system_prompt")
    return
  }

  isThinking.value = true
  chatText.value = ""

  // 1. 构造带表情选项的 system prompt
  const expressions = Array.from(exprMap.value.values()).map(e => ({
    name: e.name,
    display_name: e.display_name,
  }))
  const systemPrompt = buildCharacterPrompt(cfg.system_prompt, expressions)

  // 2. 格式化用户消息 + 追加格式提醒
  const formatReminder = "\n\n（注意：忽略角色卡中的方括号动作描写格式，严格按 system prompt 要求的 JSON 格式回复，reply 只包含说出口的话，不包含任何 [] 动作/神态描写）"
  const userMsg = (cfg.user_prompt_template
    ? cfg.user_prompt_template.replace("{message}", text)
    : text) + formatReminder

  // 3. 调用 LLM（Anthropic 格式：system 独立参数）
  const chatCtx = chatHistory.value.slice(-6)  // 保留最近 6 条上下文
  let response: LlmResponse
  try {
    response = await callLlm(
      systemPrompt,
      [
        ...chatCtx,
        { role: "user", content: userMsg },
      ],
      {
        temperature: cfg.temperature ?? 0.7,
        maxTokens: cfg.max_tokens ?? 512,
      },
    )
    console.log("[PetWindow] LLM 解析结果:", response)
  } catch (e: any) {
    console.error("[PetWindow] LLM 调用失败:", e)
    isThinking.value = false
    return
  }

  // 4. 记录历史
  chatHistory.value.push(
    { role: "user", content: userMsg },
    { role: "assistant", content: response.reply },
  )

  // 5. 切换表情
  switchExpression(response.expression)

  // 6. 自动 TTS 朗读回复
  ttsText.value = response.reply
  await speakTts()

  isThinking.value = false
}

async function speakTts() {
  const text = ttsText.value.trim()
  const expr = currentExpr.value
  const model = modelInfo.value
  if (!text || !model?.api_base_url) return

  isSpeaking.value = true
  try {
    const params = new URLSearchParams({
      text,
      text_lang: expr?.text_lang || model.text_lang || "zh",
      ref_audio_path: expr?.ref_audio_path || model.ref_audio_path || "",
      prompt_text: expr?.prompt_text || model.prompt_text || "",
      prompt_lang: expr?.prompt_lang || model.prompt_lang || "zh",
      media_type: "wav",
      streaming_mode: "2",
    })
    const res = await fetch(`${model.api_base_url}/tts?${params}`, {
      signal: AbortSignal.timeout(60000),
    })
    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: `HTTP ${res.status}` }))
      throw new Error(err.message || `请求失败 (${res.status})`)
    }

    const reader = res.body!.getReader()
    const chunks: Uint8Array[] = []
    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      chunks.push(value)
    }

    const blob = new Blob(chunks as BlobPart[], { type: "audio/wav" })
    const url = URL.createObjectURL(blob)
    if (audioRef.value) {
      audioRef.value.src = url
      audioRef.value.play().catch(() => {})
    }
    console.log(`[PetWindow] TTS 完成, ${(blob.size / 1024).toFixed(0)}KB`)
  } catch (e: any) {
    console.error("[PetWindow] TTS 失败:", e)
  } finally {
    setTimeout(() => { isSpeaking.value = false }, 500)
  }
}

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  if (appWindow) {
    try {
      appWindow.startDragging()
    } catch (err) {
      console.error("[PetWindow] startDragging failed:", err)
    }
  }
}

function onContextMenu(e: MouseEvent) {
  e.preventDefault()
  contextMenu.value = true
}

function closeContextMenu() {
  contextMenu.value = false
}

async function closeWindow() {
  if (appWindow) {
    try { await appWindow.close() } catch {}
  }
}

async function toggleAlwaysOnTop() {
  alwaysOnTop.value = !alwaysOnTop.value
  if (appWindow) {
    try { await appWindow.setAlwaysOnTop(alwaysOnTop.value) } catch {}
  }
  contextMenu.value = false
}

async function openDevTools() {
  try {
    await openPetDevtools(modelId.value)
    console.log("[PetWindow] openPetDevtools 调用成功")
  } catch (e) {
    console.error("[PetWindow] 打开开发者工具失败:", e)
  }
  contextMenu.value = false
}

function reloadPage() {
  window.location.reload()
}

onMounted(async () => {
  console.log("[PetWindow] onMounted 开始")
  pageReady.value = true

  // 强制使 html/body/#app 背景透明
  document.documentElement.style.background = "transparent"
  document.body.style.background = "transparent"
  const appEl = document.getElementById("app")
  if (appEl) appEl.style.background = "transparent"

  await initWindow()
  await loadExpressions()
  await loadModelInfo()
  await loadLlmConfig()

  document.addEventListener("click", closeContextMenu)
  console.log("[PetWindow] onMounted 完成")
})

onUnmounted(() => {
  document.removeEventListener("click", closeContextMenu)
})
</script>

<template>
  <!-- 背景点击区域 — 整个窗口可拖拽 -->
  <div
    class="pet-window"
    :class="{ 'window-ready': initialized }"
    @mousedown="onMouseDown"
    @contextmenu="onContextMenu"
  >
    <!-- 加载过程中显示简单的状态文字 -->
    <div v-if="!initialized" class="status-message">
      <div class="loading-spinner"></div>
      <div class="status-text initializing">加载角色数据...</div>
    </div>

    <!-- 立绘图片（只有在数据加载完成且无错误时显示） -->
    <template v-else>
      <div v-if="currentExpr && illustrationPath && dataUrl" class="image-wrapper">
        <img
          :src="dataUrl"
          class="character-img"
          draggable="false"
          @error="imgError = true"
        />
        <!-- 调试信息 overlay（hover 显示） -->
        <div class="debug-info">
          <div>路径: {{ illustrationPath }}</div>
          <div>窗口: {{ !!appWindow }} / {{ !!appWebviewWindow }}</div>
          <div>置顶: {{ alwaysOnTop }}</div>
        </div>
      </div>

      <!-- 数据库错误 -->
      <div v-else-if="loadError" class="status-message">
        <svg class="status-icon" viewBox="0 0 24 24" width="48" height="48">
          <path fill="#999" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>
        </svg>
        <div class="status-text error-text">{{ loadError }}</div>
        <div class="status-hint">右键点击可重新加载或打开开发者工具</div>
      </div>

      <!-- 图片加载失败（立绘路径存在但服务端不可用） -->
      <div v-else-if="currentExpr && illustrationPath && imgError" class="status-message">
        <svg class="status-icon" viewBox="0 0 24 24" width="48" height="48">
          <path fill="#e74c3c" d="M21 19V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2zM8.5 13.5l2.5 3.01L14.5 12l4.5 6H5l3.5-4.5z"/>
        </svg>
        <div class="status-text" style="color:#e74c3c">立绘加载失败</div>
        <div class="status-hint" style="color:#e74c3c;opacity:0.7">路径: {{ illustrationPath }}</div>
        <div class="status-hint">请确认 GPT-SoVITS 引擎已启动并在运行</div>
      </div>

      <!-- 占位：未设置立绘 -->
      <div v-else class="status-message">
        <svg class="status-icon" viewBox="0 0 24 24" width="64" height="64">
          <path fill="#888" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z"/>
        </svg>
        <div class="status-text">未设置立绘</div>
        <div class="status-hint">请先在角色配置中设置表情立绘</div>
      </div>
    </template>

    <!-- 右键菜单 -->
    <div v-if="contextMenu" class="context-menu" @click.stop>
      <div class="menu-item" @click="closeWindow">关闭窗口</div>
      <div class="menu-item" @click="toggleAlwaysOnTop">
        {{ alwaysOnTop ? "取消置顶" : "置顶显示" }}
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item" @click="openDevTools">打开开发者工具</div>
      <div class="menu-item" @click="reloadPage">刷新页面</div>
    </div>

    <!-- 底部操作栏 -->
    <div v-if="initialized" class="bottom-bar" @mousedown.stop>
      <div class="input-row">
        <input
          v-model="chatText"
          class="text-input"
          placeholder="和角色对话..."
          :disabled="isThinking"
          @keyup.enter="sendChat"
        />
        <button
          class="action-btn"
          :class="{ 'is-thinking': isThinking }"
          :disabled="isThinking || !chatText.trim()"
          title="发送"
          @click="sendChat"
        >
          <svg v-if="!isThinking" viewBox="0 0 24 24" width="14" height="14"><path fill="currentColor" d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/></svg>
          <svg v-else class="thinking-spin" viewBox="0 0 24 24" width="14" height="14"><path fill="currentColor" d="M12 4V2A10 10 0 0 0 2 12h2a8 8 0 0 1 8-8z"/></svg>
        </button>
      </div>
      <div class="input-row tts-row">
        <input
          v-model="ttsText"
          class="text-input"
          placeholder="TTS 测试文本..."
          @keyup.enter="speakTts"
        />
        <button
          class="action-btn primary"
          :disabled="isSpeaking"
          title="朗读"
          @click="speakTts"
        >
          <svg v-if="!isSpeaking" viewBox="0 0 24 24" width="14" height="14"><path fill="currentColor" d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/></svg>
          <svg v-else viewBox="0 0 24 24" width="14" height="14"><path fill="currentColor" d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
        </button>
      </div>
    </div>

    <audio ref="audioRef" hidden />
  </div>
</template>

<!-- 全局样式：强制透明背景 -->
<style>
html, body, #app {
  background: transparent !important;
  margin: 0 !important;
  padding: 0 !important;
}

/* 确保任何 Quasar 或第三方组件不覆盖背景 */
body::before,
body::after,
#app::before,
#app::after {
  display: none !important;
  background: transparent !important;
}
</style>

<style scoped>
.pet-window {
  width: 100vw;
  height: 100vh;
  background: transparent !important;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  -webkit-user-select: none;
  overflow: hidden;
  position: relative;
  cursor: grab;
}

.pet-window:active {
  cursor: grabbing;
}

/* 加载动画 */
.loading-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-top-color: #888;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 8px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.initializing {
  font-size: 13px;
  color: #666;
}

.image-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  position: relative;
}

.character-img {
  max-width: 100%;
  max-height: 100vh;
  object-fit: contain;
  pointer-events: none;
}

/* 调试信息 — 悬停右上角显示 */
.debug-info {
  position: absolute;
  top: 4px;
  right: 4px;
  background: rgba(0, 0, 0, 0.6);
  color: #0f0;
  font-size: 10px;
  font-family: monospace;
  padding: 4px 8px;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.2s;
  pointer-events: none;
  line-height: 1.5;
  max-width: 90%;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-wrapper:hover .debug-info {
  opacity: 1;
}

.status-message {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px;
  text-align: center;
}

.status-icon {
  opacity: 0.4;
}

.status-text {
  color: #888;
  font-size: 15px;
  font-weight: 500;
}

.error-text {
  color: #e74c3c;
  max-width: 280px;
  word-break: break-all;
}

.status-hint {
  color: #aaa;
  font-size: 12px;
}

/* 底部操作栏 */
.bottom-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(30, 30, 30, 0.75);
  backdrop-filter: blur(8px);
  padding: 6px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  z-index: 100;
}

.input-row {
  display: flex;
  gap: 4px;
}

.tts-row {
  margin-top: 2px;
}

.text-input {
  flex: 1;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  color: #eee;
  font-size: 12px;
  padding: 6px 10px;
  outline: none;
  min-width: 0;
}

.text-input::placeholder {
  color: rgba(255, 255, 255, 0.4);
}

.text-input:focus {
  border-color: rgba(255, 255, 255, 0.4);
  background: rgba(255, 255, 255, 0.15);
}

.action-btn {
  flex-shrink: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  color: #aaa;
  cursor: pointer;
  transition: all 0.15s;
}

.action-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
}

.action-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

.action-btn.is-thinking {
  color: #FF4081;
}

.thinking-spin {
  animation: spin 0.8s linear infinite;
  transform-origin: center;
}

.action-btn.primary {
  color: #FF4081;
}

.action-btn.primary:hover:not(:disabled) {
  background: rgba(255, 64, 129, 0.2);
  color: #FF80AB;
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(50, 50, 50, 0.92);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 10px;
  padding: 6px 0;
  min-width: 170px;
  z-index: 1000;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.menu-item {
  padding: 10px 20px;
  color: #eee;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.menu-item:hover {
  background: rgba(255, 255, 255, 0.1);
}

.menu-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.08);
  margin: 4px 0;
}
</style>
