<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useQuasar } from "quasar"
import { listModels } from "@/api/model"
import { startModel, stopModel, getModelStatus } from "@/api/engine"
import { openPetWindow, closePetWindow } from "@/api/pet-window"
import type { CharacterModel } from "@/types"

const $q = useQuasar()

const models = ref<CharacterModel[]>([])
const loading = ref(false)
const runningModels = ref<Set<number>>(new Set())

async function fetchModels() {
  loading.value = true
  try {
    models.value = await listModels()
    // 检查每个模型的运行状态
    for (const m of models.value) {
      if (m.id != null) {
        try {
          const running = await getModelStatus(m.id)
          if (running) runningModels.value.add(m.id)
        } catch { /* 忽略 */ }
      }
    }
  } catch (e) {
    $q.notify({ type: "negative", message: `加载模型失败: ${e}` })
  } finally {
    loading.value = false
  }
}

async function handleStart(model: CharacterModel) {
  if (model.id == null) return
  try {
    const msg = await startModel(model.id)
    runningModels.value.add(model.id)
    // 启动后打开宠物窗口 — 用前端 origin 构造 URL 避免 WebviewUrl::App 在子窗口的解析问题
    const baseUrl = window.location.origin
    const petUrl = `${baseUrl}/pet-window/${model.id}`
    openPetWindow(model.id, petUrl).catch((err) => {
      console.warn("创建宠物窗口失败（可忽略，稍后可重试）:", err)
    })
    $q.notify({ type: "positive", message: msg })
  } catch (e) {
    $q.notify({ type: "negative", message: `启动失败: ${e}` })
  }
}

async function handleStop(model: CharacterModel) {
  if (model.id == null) return
  try {
    const msg = await stopModel(model.id)
    runningModels.value.delete(model.id)
    // 停止后关闭宠物窗口
    closePetWindow(model.id).catch(() => {})
    $q.notify({ type: "info", message: msg })
  } catch (e) {
    $q.notify({ type: "negative", message: `停止失败: ${e}` })
  }
}

function isRunning(model: CharacterModel): boolean {
  return model.id != null && runningModels.value.has(model.id)
}

/** 从 api_base_url 提取端口显示 */
function extractPort(url: string): string {
  try {
    const cleaned = url.replace(/^https?:\/\//, "").split("/")[0]
    const parts = cleaned.split(":")
    return parts[1] || "9880"
  } catch {
    return "9880"
  }
}

onMounted(fetchModels)
</script>

<template>
  <div class="q-pa-lg">
    <!-- 页面标题 -->
    <div class="row items-center justify-between q-mb-lg">
      <div>
        <div class="text-h5 text-weight-medium q-mb-xs">
          <q-icon name="pets" color="primary" class="q-mr-sm" />
          我的桌宠
        </div>
        <q-breadcrumbs class="text-grey-6 q-mt-sm">
          <q-breadcrumbs-el label="桌宠配置" />
        </q-breadcrumbs>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-center q-py-xl">
      <q-spinner color="primary" size="40px" />
      <div class="q-mt-sm text-grey-6">加载中...</div>
    </div>

    <!-- 模型卡片列表 -->
    <div v-else-if="models.length > 0" class="row q-col-gutter-lg">
      <div
        v-for="m in models"
        :key="m.id ?? 0"
        class="col-12 col-sm-6 col-md-4"
      >
        <q-card flat bordered class="pet-card">
          <!-- 卡片顶部：头像 + 名称 + 状态 -->
          <q-card-section class="text-center q-pb-none">
            <div
              class="avatar-circle q-mb-sm"
              :class="{ 'avatar-running': isRunning(m) }"
            >
              <q-icon
                name="smart_toy"
                size="40px"
                color="white"
              />
            </div>
            <div class="text-h6 text-weight-medium">{{ m.name }}</div>
            <q-badge
              :color="isRunning(m) ? 'positive' : 'grey-5'"
              class="q-mt-xs"
            >
              <q-icon
                :name="isRunning(m) ? 'play_arrow' : 'stop'"
                size="14px"
                class="q-mr-xs"
              />
              {{ isRunning(m) ? "运行中" : "已停止" }}
            </q-badge>
          </q-card-section>

          <!-- 卡片主体：模型信息 -->
          <q-card-section>
            <q-separator class="q-mb-sm" />

            <div class="text-caption text-grey-7 q-mb-sm">
              <div class="row items-center q-mb-xs">
                <q-icon name="folder" size="16px" class="q-mr-xs" />
                <span class="ellipsis" :title="m.model_start_path">
                  {{ m.model_start_path || "-" }}
                </span>
              </div>
              <div class="row items-center q-mb-xs">
                <q-icon name="link" size="16px" class="q-mr-xs" />
                <span>API: {{ m.api_base_url || `http://0.0.0.0:${extractPort(m.api_base_url)}` }}</span>
              </div>
              <div class="row items-center q-mb-xs">
                <q-icon name="model_training" size="16px" class="q-mr-xs" />
                <span class="ellipsis" :title="m.gpt_model">GPT: {{ m.gpt_model || "-" }}</span>
              </div>
              <div class="row items-center">
                <q-icon name="graphic_eq" size="16px" class="q-mr-xs" />
                <span class="ellipsis" :title="m.sovits_model">SoVITS: {{ m.sovits_model || "-" }}</span>
              </div>
            </div>

            <q-separator class="q-mb-sm" />

            <!-- 参考音频和提示文本 -->
            <div class="text-caption text-grey-7">
              <div class="q-mb-xs">
                <span class="text-grey-5">参考音频:</span>
                <span class="ellipsis block" :title="m.ref_audio_path">{{ m.ref_audio_path || "-" }}</span>
              </div>
              <div>
                <span class="text-grey-5">提示文本:</span>
                <span class="ellipsis block" :title="m.prompt_text">{{ m.prompt_text || "-" }}</span>
              </div>
            </div>
          </q-card-section>

          <!-- 卡片底部：操作按钮 -->
          <q-card-actions class="row justify-around q-pa-md">
            <q-btn
              flat
              round
              :icon="isRunning(m) ? 'stop' : 'play_arrow'"
              :color="isRunning(m) ? 'negative' : 'positive'"
              size="md"
              :loading="false"
              @click="isRunning(m) ? handleStop(m) : handleStart(m)"
            >
              <q-tooltip>{{ isRunning(m) ? "停止" : "启动" }}</q-tooltip>
            </q-btn>
            <q-btn
              flat
              round
              icon="edit"
              color="primary"
              size="md"
              :to="`/character-edit/${m.id}`"
            >
              <q-tooltip>详细配置</q-tooltip>
            </q-btn>
          </q-card-actions>
        </q-card>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-else
      class="absolute-center text-center text-grey-5"
    >
      <q-icon name="smart_toy" size="64px" />
      <div class="text-h6 q-mt-md">还没有角色模型</div>
      <div class="text-body2 q-mt-sm q-mb-md">先去角色模型配置页面创建模型吧</div>
      <q-btn
        label="去配置"
        color="primary"
        icon="add"
        unelevated
        to="/character-config"
      />
    </div>
  </div>
</template>

<style scoped>
.pet-card {
  border-radius: 12px;
  transition: transform 0.2s, box-shadow 0.2s;
}

.pet-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.12);
}

.avatar-circle {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #FF4081, #C51162);
  border: 3px solid rgba(255, 255, 255, 0.6);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  transition: box-shadow 0.3s;
}

.avatar-running {
  box-shadow: 0 0 20px rgba(76, 175, 80, 0.6);
  border-color: #4CAF50;
}

.ellipsis {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
