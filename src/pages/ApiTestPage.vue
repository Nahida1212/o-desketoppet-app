<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useQuasar } from "quasar"
import { listModels } from "@/api/model"
import { getModelStatus } from "@/api/engine"
import type { CharacterModel } from "@/types"

const $q = useQuasar()

// ── Connection ──────────────────────────────────────────────
const models = ref<CharacterModel[]>([])
const selectedModel = ref<CharacterModel | null>(null)
const manualUrl = ref("http://127.0.0.1:9880")
const useManualUrl = ref(false)

const apiUrl = computed(() => {
  const raw = useManualUrl.value
    ? manualUrl.value
    : selectedModel.value?.api_base_url ?? "http://127.0.0.1:9880"
  let url = raw.trim()
  if (url && !/^https?:\/\//i.test(url)) url = `http://${url}`
  return url || "http://127.0.0.1:9880"
})

const connectionOk = ref<boolean | null>(null)
const connectionChecking = ref(false)

async function checkConnection() {
  connectionChecking.value = true
  connectionOk.value = null
  try {
    const res = await fetch(`${apiUrl.value}/list_models`, { signal: AbortSignal.timeout(5000) })
    connectionOk.value = res.ok
    if (res.ok) {
      const data = await res.json()
      availableGptWeights.value = data.gpt_weights ?? []
      availableSovitsWeights.value = data.sovits_weights ?? []
      currentGptWeight.value = data.current?.gpt ?? ""
      currentSovitsWeight.value = data.current?.sovits ?? ""
    }
  } catch {
    connectionOk.value = false
  } finally {
    connectionChecking.value = false
  }
}

// ── TTS Form ────────────────────────────────────────────────
const ttsText = ref("你好，欢迎使用GPT-SoVITS语音合成系统。")
const ttsTextLang = ref("zh")
const ttsRefAudio = ref("")
const ttsPromptText = ref("")
const ttsPromptLang = ref("zh")
const ttsSplitMethod = ref("cut5")
const ttsMediaType = ref("wav")
const ttsStreaming = ref(false)
const ttsSpeed = ref(1.0)
const ttsTemperature = ref(1.0)
const ttsTopK = ref(10)
const ttsTopP = ref(1.0)
const ttsSeed = ref(-1)
const ttsBatchSize = ref(1)
const ttsRepPenalty = ref(1.35)
const ttsSampleSteps = ref(8)

const languages = ["zh", "en", "ja", "ko", "yue"]
const cutMethods = ["cut0", "cut1", "cut2", "cut3", "cut4", "cut5"]
const mediaTypes = ["wav", "ogg", "aac", "raw"]

const isGenerating = ref(false)
const audioUrl = ref<string | null>(null)
const audioBlob = ref<Blob | null>(null)
const lastError = ref<string | null>(null)
const responseTime = ref<number | null>(null)
const audioRef = ref<HTMLAudioElement | null>(null)

// 流式进度
const streamingChunks = ref(0)
const streamingBytes = ref(0)

/** 尝试自动播放 audio，静默处理浏览器策略阻止 */
function tryAutoPlay() {
  const el = audioRef.value
  if (!el) return
  el.play().catch(() => {
    // 浏览器阻止自动播放（未与页面交互时），忽略
  })
}

async function testTts() {
  if (!ttsText.value) {
    $q.notify({ type: "warning", message: "请输入合成文本" })
    return
  }
  if (!ttsRefAudio.value) {
    $q.notify({ type: "warning", message: "请输入参考音频路径" })
    return
  }

  isGenerating.value = true
  lastError.value = null
  audioUrl.value = null
  audioBlob.value = null
  streamingChunks.value = 0
  streamingBytes.value = 0

  if (audioUrl.value) URL.revokeObjectURL(audioUrl.value)

  const start = Date.now()

  try {
    const params = new URLSearchParams({
      text: ttsText.value,
      text_lang: ttsTextLang.value,
      ref_audio_path: ttsRefAudio.value,
      prompt_text: ttsPromptText.value,
      prompt_lang: ttsPromptLang.value,
      text_split_method: ttsSplitMethod.value,
      media_type: ttsMediaType.value,
      speed_factor: ttsSpeed.value.toString(),
      temperature: ttsTemperature.value.toString(),
      top_k: ttsTopK.value.toString(),
      top_p: ttsTopP.value.toString(),
      seed: ttsSeed.value.toString(),
      batch_size: ttsBatchSize.value.toString(),
      repetition_penalty: ttsRepPenalty.value.toString(),
      sample_steps: ttsSampleSteps.value.toString(),
    })
    if (ttsStreaming.value) params.set("streaming_mode", "2")  // 2=真实流式, 会在返回第一块音频时立即开始播放

    const res = await fetch(`${apiUrl.value}/tts?${params}`, {
      signal: AbortSignal.timeout(180000),
    })

    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: `HTTP ${res.status}` }))
      throw new Error(err.message || err.Exception || `请求失败 (${res.status})`)
    }

    if (ttsStreaming.value && res.body) {
      // ── 流式模式：逐块读取，渐进更新 audio ──
      const reader = res.body.getReader()
      const chunks: Uint8Array[] = []

      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        chunks.push(value)
        streamingChunks.value++
        streamingBytes.value += value.length

        // 每收到一块就更新 audio 播放器，让用户渐进听到声音
        const partialBlob = new Blob(chunks as BlobPart[], { type: `audio/${ttsMediaType.value}` })
        if (audioUrl.value) URL.revokeObjectURL(audioUrl.value)
        audioUrl.value = URL.createObjectURL(partialBlob)
        tryAutoPlay()
      }

      audioBlob.value = new Blob(chunks as BlobPart[], { type: `audio/${ttsMediaType.value}` })
      responseTime.value = Date.now() - start

      $q.notify({
        type: "positive",
        message: `合成成功（流式 ${streamingChunks.value} 块, ${(responseTime.value / 1000).toFixed(1)}s, ${(audioBlob.value.size / 1024).toFixed(0)}KB）`,
      })
    } else {
      // ── 非流式模式：一次性等完整音频 ──
      const blob = await res.blob()
      audioBlob.value = blob
      audioUrl.value = URL.createObjectURL(blob)
      responseTime.value = Date.now() - start
      tryAutoPlay()

      $q.notify({
        type: "positive",
        message: `合成成功 (${(responseTime.value / 1000).toFixed(1)}s, ${(blob.size / 1024).toFixed(0)}KB)`,
      })
    }
  } catch (e: any) {
    lastError.value = e.message ?? String(e)
    $q.notify({ type: "negative", message: `合成失败: ${lastError.value}` })
  } finally {
    isGenerating.value = false
  }
}

function downloadAudio() {
  if (!audioBlob.value || !audioUrl.value) return
  const a = document.createElement("a")
  a.href = audioUrl.value
  a.download = `tts_output.${ttsMediaType.value}`
  a.click()
}

// ── Model Switching ─────────────────────────────────────────
const availableGptWeights = ref<string[]>([])
const availableSovitsWeights = ref<string[]>([])
const currentGptWeight = ref("")
const currentSovitsWeight = ref("")

const switchGptPath = ref("")
const switchSovitsPath = ref("")
const switchingGpt = ref(false)
const switchingSovits = ref(false)

async function setGptWeights() {
  if (!switchGptPath.value) {
    $q.notify({ type: "warning", message: "请输入 GPT 权重路径" })
    return
  }
  switchingGpt.value = true
  try {
    const res = await fetch(`${apiUrl.value}/set_gpt_weights?weights_path=${encodeURIComponent(switchGptPath.value)}`, {
      signal: AbortSignal.timeout(10000),
    })
    const data = await res.json()
    if (res.ok) {
      currentGptWeight.value = switchGptPath.value
      $q.notify({ type: "positive", message: "GPT 模型切换成功" })
    } else {
      throw new Error(data.message || data.Exception || "切换失败")
    }
  } catch (e: any) {
    $q.notify({ type: "negative", message: `切换失败: ${e.message ?? e}` })
  } finally {
    switchingGpt.value = false
  }
}

async function setSovitsWeights() {
  if (!switchSovitsPath.value) {
    $q.notify({ type: "warning", message: "请输入 SoVITS 权重路径" })
    return
  }
  switchingSovits.value = true
  try {
    const res = await fetch(`${apiUrl.value}/set_sovits_weights?weights_path=${encodeURIComponent(switchSovitsPath.value)}`, {
      signal: AbortSignal.timeout(10000),
    })
    const data = await res.json()
    if (res.ok) {
      currentSovitsWeight.value = switchSovitsPath.value
      $q.notify({ type: "positive", message: "SoVITS 模型切换成功" })
    } else {
      throw new Error(data.message || data.Exception || "切换失败")
    }
  } catch (e: any) {
    $q.notify({ type: "negative", message: `切换失败: ${e.message ?? e}` })
  } finally {
    switchingSovits.value = false
  }
}

// ── Reference Audio ─────────────────────────────────────────
const setRefAudioPath = ref("")
const settingRefAudio = ref(false)

async function setReferAudio() {
  if (!setRefAudioPath.value) {
    $q.notify({ type: "warning", message: "请输入参考音频路径" })
    return
  }
  settingRefAudio.value = true
  try {
    const res = await fetch(`${apiUrl.value}/set_refer_audio?refer_audio_path=${encodeURIComponent(setRefAudioPath.value)}`, {
      signal: AbortSignal.timeout(10000),
    })
    const data = await res.json()
    if (res.ok) {
      $q.notify({ type: "positive", message: "参考音频切换成功" })
    } else {
      throw new Error(data.message || data.Exception || "切换失败")
    }
  } catch (e: any) {
    $q.notify({ type: "negative", message: `切换失败: ${e.message ?? e}` })
  } finally {
    settingRefAudio.value = false
  }
}

// ── Control ─────────────────────────────────────────────────
const controlling = ref(false)

async function sendCommand(command: "restart" | "exit") {
  const label = command === "restart" ? "重启" : "退出"
  $q.dialog({
    title: `确认${label}?`,
    message: `将要向引擎发送 "${command}" 命令，确认?`,
    cancel: true,
    persistent: true,
  }).onOk(async () => {
    controlling.value = true
    try {
      const res = await fetch(`${apiUrl.value}/control?command=${command}`, {
        signal: AbortSignal.timeout(5000),
      })
      if (res.ok) {
        $q.notify({ type: "info", message: `${label}命令已发送` })
      } else {
        const data = await res.json().catch(() => ({}))
        throw new Error(data.message || `HTTP ${res.status}`)
      }
    } catch (e: any) {
      // For "exit", the connection drops immediately — that's normal
      if (command === "exit" && e.name === "TypeError") {
        $q.notify({ type: "info", message: "退出命令已发送" })
      } else {
        $q.notify({ type: "negative", message: `${label}失败: ${e.message ?? e}` })
      }
    } finally {
      controlling.value = false
    }
  })
}

// ── Lifecycle ───────────────────────────────────────────────
onMounted(async () => {
  try {
    const all = await listModels()
    const running: CharacterModel[] = []
    for (const m of all) {
      if (m.id != null) {
        try {
          const ok = await getModelStatus(m.id)
          if (ok) running.push(m)
        } catch { /* skip */ }
      }
    }
    models.value = running
    if (running.length > 0) {
      selectedModel.value = running[0]
      // Also extract ref_audio from the model to pre-fill
      if (running[0].ref_audio_path) {
        ttsRefAudio.value = running[0].ref_audio_path
        ttsPromptText.value = running[0].prompt_text
        ttsPromptLang.value = running[0].prompt_lang
      }
      // Auto-check connection
      setTimeout(checkConnection, 300)
    }
  } catch { /* ignore */ }
})
</script>

<template>
  <div class="q-pa-lg">
    <div class="text-h5 text-weight-medium q-mb-lg">
      <q-icon name="science" color="primary" class="q-mr-sm" />
      API 测试
    </div>

    <!-- ── Connection 卡 ── -->
    <q-card flat bordered class="q-mb-lg">
      <q-card-section>
        <div class="text-h6 q-mb-sm">API 连接</div>
        <div class="row items-end q-col-gutter-md">
          <div class="col-12 col-md-5">
            <q-select
              v-model="selectedModel"
              :options="models"
              option-label="name"
              label="运行中的模型"
              clearable
              :disable="useManualUrl"
              @update:model-value="useManualUrl = false"
            >
              <template #option="{ opt }">
                <q-item v-bind="$props" clickable>
                  <q-item-section avatar>
                    <q-icon name="smart_toy" color="primary" />
                  </q-item-section>
                  <q-item-section>
                    <q-item-label>{{ opt.name }}</q-item-label>
                    <q-item-label caption>{{ opt.api_base_url }}</q-item-label>
                  </q-item-section>
                </q-item>
              </template>
            </q-select>
          </div>
          <div class="col-auto text-grey-6 text-center q-pb-md">或</div>
          <div class="col-12 col-md-4">
            <q-input
              v-model="manualUrl"
              label="手动输入 API URL"
              placeholder="http://127.0.0.1:9880"
              :disable="selectedModel != null"
              @focus="useManualUrl = true"
            />
          </div>
          <div class="col-12 col-md-auto">
            <q-btn
              unelevated
              color="primary"
              :loading="connectionChecking"
              :icon="connectionOk === true ? 'check_circle' : connectionOk === false ? 'error' : 'wifi'"
              :label="connectionOk === true ? '已连接' : connectionOk === false ? '连接失败' : '测试连接'"
              @click="checkConnection"
            />
          </div>
        </div>
      </q-card-section>
      <q-card-section v-if="connectionOk" class="q-pt-none">
        <div class="text-caption text-grey-7 row q-col-gutter-x-lg">
          <div><b>当前 GPT:</b> {{ currentGptWeight || "-" }}</div>
          <div><b>当前 SoVITS:</b> {{ currentSovitsWeight || "-" }}</div>
        </div>
      </q-card-section>
    </q-card>

    <!-- ── TTS 测试卡 ── -->
    <q-card flat bordered class="q-mb-lg">
      <q-card-section>
        <div class="text-h6 q-mb-sm">TTS 语音合成测试</div>
      </q-card-section>

      <q-card-section class="q-pt-none">
        <div class="row q-col-gutter-md">
          <!-- 左侧：主要参数 -->
          <div class="col-12 col-md-8">
            <q-input
              v-model="ttsText"
              label="合成文本 *"
              placeholder="输入要合成的文本"
              type="textarea"
              :rows="3"
              class="q-mb-md"
            />

            <div class="row q-col-gutter-md">
              <div class="col-6">
                <q-select
                  v-model="ttsTextLang"
                  :options="languages"
                  label="文本语言 *"
                  class="q-mb-md"
                />
              </div>
              <div class="col-6">
                <q-select
                  v-model="ttsMediaType"
                  :options="mediaTypes"
                  label="输出格式"
                  class="q-mb-md"
                />
              </div>
            </div>

            <q-input
              v-model="ttsRefAudio"
              label="参考音频路径 *"
              placeholder="引擎内的完整路径，如: default_ref.wav"
              class="q-mb-md"
            />

            <div class="row q-col-gutter-md">
              <div class="col-8">
                <q-input
                  v-model="ttsPromptText"
                  label="提示文本（参考音频原文本）"
                  placeholder="参考音频对应的文本"
                  class="q-mb-md"
                />
              </div>
              <div class="col-4">
                <q-select
                  v-model="ttsPromptLang"
                  :options="languages"
                  label="提示文本语言"
                  class="q-mb-md"
                />
              </div>
            </div>
          </div>

          <!-- 右侧：操作按钮 + 状态 -->
          <div class="col-12 col-md-4 column items-center justify-center">
            <q-btn
              unelevated
              color="primary"
              size="lg"
              class="full-width q-mb-sm"
              :loading="isGenerating"
              :disable="!connectionOk"
              icon="play_arrow"
              label="生成语音"
              @click="testTts"
            />
            <div v-if="isGenerating && ttsStreaming" class="text-caption text-primary row items-center q-mt-xs">
              <q-spinner size="12px" class="q-mr-xs" />
              接收中: {{ streamingChunks }} 块 / {{ (streamingBytes / 1024).toFixed(0) }} KB
            </div>
            <div v-if="responseTime" class="text-caption text-grey-6">
              {{ (responseTime / 1000).toFixed(1) }}s
              <template v-if="audioBlob"> / {{ (audioBlob.size / 1024).toFixed(0) }} KB</template>
            </div>
            <div v-if="lastError" class="text-caption text-negative q-mt-xs text-center">
              {{ lastError }}
            </div>
          </div>
        </div>
      </q-card-section>

      <!-- 高级参数 -->
      <q-card-section class="q-pt-none">
        <q-expansion-item
          group="tts-advanced"
          icon="tune"
          label="高级参数"
          header-class="text-grey-7"
        >
          <div class="row q-col-gutter-md q-mt-sm">
            <div class="col-6 col-md-3">
              <q-select v-model="ttsSplitMethod" :options="cutMethods" label="切分方式" />
            </div>
            <div class="col-6 col-md-2">
              <q-input v-model.number="ttsSpeed" label="语速" type="number" step="0.1" min="0.5" max="2.0" />
            </div>
            <div class="col-6 col-md-2">
              <q-input v-model.number="ttsTemperature" label="温度" type="number" step="0.1" min="0.1" max="2.0" />
            </div>
            <div class="col-6 col-md-2">
              <q-input v-model.number="ttsTopK" label="Top-K" type="number" step="1" min="1" />
            </div>
            <div class="col-6 col-md-2">
              <q-input v-model.number="ttsTopP" label="Top-P" type="number" step="0.05" min="0" max="1" />
            </div>
            <div class="col-6 col-md-2">
              <q-input v-model.number="ttsSeed" label="随机种子" type="number" />
            </div>
            <div class="col-6 col-md-2">
              <q-input v-model.number="ttsBatchSize" label="Batch Size" type="number" min="1" />
            </div>
            <div class="col-6 col-md-3">
              <q-input v-model.number="ttsRepPenalty" label="重复惩罚" type="number" step="0.1" min="1.0" />
            </div>
            <div class="col-6 col-md-2">
              <q-input v-model.number="ttsSampleSteps" label="采样步数" type="number" min="1" />
            </div>
            <div class="col-6 col-md-2 flex items-center">
              <q-toggle v-model="ttsStreaming" label="流式模式" />
            </div>
          </div>
        </q-expansion-item>
      </q-card-section>

      <!-- 音频播放 -->
      <q-card-section v-if="audioUrl" class="q-pt-none">
        <q-separator class="q-mb-md" />
        <div class="text-subtitle2 q-mb-sm">合成结果</div>
        <div class="row items-center q-col-gutter-md">
          <div class="col">
            <audio
              ref="audioRef"
              :src="audioUrl"
              controls
              class="full-width"
              style="height: 44px"
            />
          </div>
          <div class="col-auto">
            <q-btn
              flat
              color="primary"
              icon="download"
              label="下载"
              @click="downloadAudio"
            />
          </div>
        </div>
      </q-card-section>
    </q-card>

    <!-- ── 模型管理卡 ── -->
    <q-card flat bordered class="q-mb-lg">
      <q-card-section>
        <div class="text-h6 q-mb-sm">模型管理</div>
      </q-card-section>

      <q-card-section class="q-pt-none">
        <div class="text-subtitle2 q-mb-sm text-grey-7">可用权重</div>
        <div class="row q-col-gutter-md">
          <div class="col-12 col-md-6">
            <q-list dense bordered separator class="rounded-borders">
              <q-item class="bg-grey-2">
                <q-item-section>GPT 权重</q-item-section>
              </q-item>
              <q-item v-if="availableGptWeights.length === 0">
                <q-item-section class="text-grey-5 text-caption">暂无数据，请先测试连接</q-item-section>
              </q-item>
              <q-item v-for="w in availableGptWeights" :key="w" dense>
                <q-item-section>
                  <q-item-label class="text-caption">{{ w }}</q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </div>
          <div class="col-12 col-md-6">
            <q-list dense bordered separator class="rounded-borders">
              <q-item class="bg-grey-2">
                <q-item-section>SoVITS 权重</q-item-section>
              </q-item>
              <q-item v-if="availableSovitsWeights.length === 0">
                <q-item-section class="text-grey-5 text-caption">暂无数据，请先测试连接</q-item-section>
              </q-item>
              <q-item v-for="w in availableSovitsWeights" :key="w" dense>
                <q-item-section>
                  <q-item-label class="text-caption">{{ w }}</q-item-label>
                </q-item-section>
              </q-item>
            </q-list>
          </div>
        </div>

        <q-separator class="q-my-md" />

        <div class="text-subtitle2 q-mb-sm text-grey-7">切换 GPT 模型</div>
        <div class="row items-end q-col-gutter-md">
          <div class="col-12 col-md-6">
            <q-input
              v-model="switchGptPath"
              label="GPT 权重路径"
              :placeholder="`如: GPT_weights_v2Pro/${currentGptWeight || 'xxx.ckpt'}`"
            />
          </div>
          <div class="col-auto">
            <q-btn
              unelevated
              color="primary"
              :loading="switchingGpt"
              :disable="!connectionOk"
              label="切换"
              @click="setGptWeights"
            />
          </div>
        </div>

        <q-separator class="q-my-md" />

        <div class="text-subtitle2 q-mb-sm text-grey-7">切换 SoVITS 模型</div>
        <div class="row items-end q-col-gutter-md">
          <div class="col-12 col-md-6">
            <q-input
              v-model="switchSovitsPath"
              label="SoVITS 权重路径"
              :placeholder="`如: SoVITS_weights_v2Pro/${currentSovitsWeight || 'xxx.pth'}`"
            />
          </div>
          <div class="col-auto">
            <q-btn
              unelevated
              color="primary"
              :loading="switchingSovits"
              :disable="!connectionOk"
              label="切换"
              @click="setSovitsWeights"
            />
          </div>
        </div>
      </q-card-section>
    </q-card>

    <!-- ── 参考音频 卡 ── -->
    <q-card flat bordered class="q-mb-lg">
      <q-card-section>
        <div class="text-h6 q-mb-sm">参考音频</div>
      </q-card-section>
      <q-card-section class="q-pt-none">
        <div class="row items-end q-col-gutter-md">
          <div class="col-12 col-md-6">
            <q-input
              v-model="setRefAudioPath"
              label="参考音频路径"
              placeholder="如: default_ref.wav 或引擎内完整路径"
            />
          </div>
          <div class="col-auto">
            <q-btn
              unelevated
              color="primary"
              :loading="settingRefAudio"
              :disable="!connectionOk"
              label="切换参考音频"
              @click="setReferAudio"
            />
          </div>
        </div>
      </q-card-section>
    </q-card>

    <!-- ── 控制卡 ── -->
    <q-card flat bordered>
      <q-card-section>
        <div class="text-h6 q-mb-sm">引擎控制</div>
      </q-card-section>
      <q-card-section class="q-pt-none">
        <div class="row q-col-gutter-md">
          <div class="col">
            <q-btn
              unelevated
              color="warning"
              class="full-width"
              :disable="!connectionOk || controlling"
              icon="restart_alt"
              label="重启引擎"
              @click="sendCommand('restart')"
            />
          </div>
          <div class="col">
            <q-btn
              unelevated
              color="negative"
              class="full-width"
              :disable="!connectionOk || controlling"
              icon="power_settings_new"
              label="退出引擎"
              @click="sendCommand('exit')"
            />
          </div>
        </div>
      </q-card-section>
    </q-card>
  </div>
</template>

<style scoped>
audio {
  border-radius: 8px;
}
</style>
