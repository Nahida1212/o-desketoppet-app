<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useRouter, useRoute } from "vue-router"
import { useQuasar } from "quasar"
import { open } from "@tauri-apps/plugin-dialog"
import { getModel, updateModel } from "@/api/model"
import { listExpressions, updateExpression } from "@/api/expression"
import { getLlmConfig, saveLlmConfig, ensureDefaultExpressions } from "@/api/llm"
import type { CharacterModel, CharacterExpression, CharacterLlmConfig } from "@/types"

const $q = useQuasar()
const router = useRouter()
const route = useRoute()

const modelId = computed(() => Number(route.params.id))
const loading = ref(true)
const tab = ref("basic")

// ── 基础信息 ──
const model = ref<CharacterModel | null>(null)
const basicForm = ref({
  name: "",
  model_start_path: "",
  api_base_url: "",
  gpt_model: "",
  sovits_model: "",
  config_path: "",
})
const savingBasic = ref(false)

// ── 角色差分（固定 5 种表情） ──
const EXPR_META = [
  { name: "default",    display_name: "常态",     icon: "face" },
  { name: "happy",      display_name: "高兴",     icon: "mood" },
  { name: "angry",      display_name: "生气",     icon: "sentiment_very_dissatisfied" },
  { name: "sad",        display_name: "悲伤",     icon: "sentiment_dissatisfied" },
  { name: "surprised",  display_name: "惊讶",     icon: "sentiment_surprised" },
] as const

/** key → 数据库记录 的映射 */
const exprMap = ref<Map<string, CharacterExpression>>(new Map())

const exprDialog = ref(false)
const editingExpr = ref<CharacterExpression | null>(null)
const editingMeta = ref<typeof EXPR_META[number] | null>(null)
const exprForm = ref({
  ref_audio_path: "",
  prompt_text: "",
  prompt_lang: "zh" as string,
  text_lang: "zh" as string,
  illustration_path: "",
})
const savingExpr = ref(false)

// ── LLM 提示词 ──
const llmConfig = ref<CharacterLlmConfig>({
  model_id: 0,
  system_prompt: "",
  user_prompt_template: "",
  greeting_message: "",
  temperature: 0.7,
  max_tokens: 1024,
  top_p: 0.9,
  presence_penalty: 0.0,
  frequency_penalty: 0.0,
})
const savingLlm = ref(false)

const LANG_OPTIONS = ["zh", "en", "ja", "ko", "yue"]

/** 从 exprMap 按 key 取记录 */
function getExpr(key: string): CharacterExpression | undefined {
  return exprMap.value.get(key)
}

// ── 初始化加载 ──
async function loadAll() {
  loading.value = true
  try {
    const m = await getModel(modelId.value)
    if (!m) {
      $q.notify({ type: "negative", message: "角色不存在" })
      router.push("/character-config")
      return
    }
    model.value = m
    basicForm.value = {
      name: m.name,
      model_start_path: m.model_start_path,
      api_base_url: m.api_base_url,
      gpt_model: m.gpt_model,
      sovits_model: m.sovits_model,
      config_path: m.config_path,
    }

    // 确保 5 个默认差分存在（旧模型迁移）
    await ensureDefaultExpressions(modelId.value)

    const [exprs, llm] = await Promise.all([
      listExpressions(modelId.value),
      getLlmConfig(modelId.value).catch(() => null),
    ])
    // 建立 key → 记录 映射
    const map = new Map<string, CharacterExpression>()
    for (const e of exprs) {
      map.set(e.name, e)
    }
    exprMap.value = map

    if (llm) llmConfig.value = llm
    else llmConfig.value.model_id = modelId.value
  } catch (e: any) {
    $q.notify({ type: "negative", message: `加载失败: ${e}` })
  } finally {
    loading.value = false
  }
}

// ── 保存基础信息 ──
async function saveBasic() {
  if (!basicForm.value.name.trim()) {
    $q.notify({ type: "warning", message: "请输入角色名称" })
    return
  }
  savingBasic.value = true
  try {
    await updateModel(modelId.value, {
      ...basicForm.value,
      ref_audio_path: "", prompt_text: "", prompt_lang: "zh", text_lang: "zh",
    })
    $q.notify({ type: "positive", message: "基本设置已保存" })
  } catch (e: any) {
    $q.notify({ type: "negative", message: `保存失败: ${e}` })
  } finally {
    savingBasic.value = false
  }
}

/** 文件选择器 */
async function pickFile(field: string) {
  const filters: Record<string, { name: string; extensions: string[] }[]> = {
    model_start_path: [{ name: "启动文件", extensions: ["bat", "sh", "py", "exe"] }],
    gpt_model: [{ name: "GPT 模型", extensions: ["pth", "pt", "ckpt"] }],
    sovits_model: [{ name: "SoVITS 模型", extensions: ["pth", "pt", "ckpt"] }],
    config_path: [{ name: "配置文件", extensions: ["json", "yaml", "yml", "toml"] }],
    ref_audio_path: [{ name: "音频文件", extensions: ["wav", "mp3", "flac", "ogg", "m4a", "aac"] }],
    illustration_path: [{ name: "图片文件", extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"] }],
  }
  const selected = await open({
    multiple: false,
    directory: false,
    filters: filters[field] ?? [{ name: "所有文件", extensions: ["*"] }],
  })
  if (!selected) return
  ;(exprForm.value as any)[field] = selected
}

// ── 差分编辑 ──
/** 点击卡片 → 弹出文件选择器选立绘，选中后自动保存 */
async function pickIllustration(name: string) {
  const expr = exprMap.value.get(name)
  if (!expr) return
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "图片文件", extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"] }],
  })
  if (!selected) return
  try {
    await updateExpression(expr.id!, {
      model_id: modelId.value,
      name: expr.name,
      display_name: expr.display_name,
      sort_order: expr.sort_order,
      ref_audio_path: expr.ref_audio_path,
      prompt_text: expr.prompt_text,
      prompt_lang: expr.prompt_lang,
      text_lang: expr.text_lang,
      illustration_path: selected,
    })
    // 更新本地映射
    exprMap.value.set(name, { ...expr, illustration_path: selected })
    const displayName = EXPR_META.find(m => m.name === name)?.display_name || name
    $q.notify({ type: "positive", message: `${displayName} 立绘已更新` })
  } catch (e: any) {
    $q.notify({ type: "negative", message: `保存失败: ${e}` })
  }
}

function openEditExpr(name: string) {
  const meta = EXPR_META.find(m => m.name === name)
  if (!meta) return
  const expr = exprMap.value.get(name)
  if (!expr) return
  editingMeta.value = meta
  editingExpr.value = expr
  exprForm.value = {
    ref_audio_path: expr.ref_audio_path,
    prompt_text: expr.prompt_text,
    prompt_lang: expr.prompt_lang,
    text_lang: expr.text_lang,
    illustration_path: expr.illustration_path,
  }
  exprDialog.value = true
}

async function saveExpr() {
  const expr = editingExpr.value
  if (!expr || !expr.id) return
  savingExpr.value = true
  try {
    await updateExpression(expr.id, {
      model_id: modelId.value,
      name: expr.name,
      display_name: expr.display_name,
      sort_order: expr.sort_order,
      ...exprForm.value,
    })
    $q.notify({ type: "positive", message: `${editingMeta.value?.display_name} 已保存` })
    exprDialog.value = false
    // 刷新列表
    const exprs = await listExpressions(modelId.value)
    const map = new Map<string, CharacterExpression>()
    for (const e of exprs) {
      map.set(e.name, e)
    }
    exprMap.value = map
  } catch (e: any) {
    $q.notify({ type: "negative", message: `保存失败: ${e}` })
  } finally {
    savingExpr.value = false
  }
}

// ── LLM 提示词 ──
async function saveLlm() {
  savingLlm.value = true
  try {
    await saveLlmConfig(modelId.value, { ...llmConfig.value })
    $q.notify({ type: "positive", message: "LLM 提示词配置已保存" })
  } catch (e: any) {
    $q.notify({ type: "negative", message: `保存失败: ${e}` })
  } finally {
    savingLlm.value = false
  }
}

onMounted(loadAll)
</script>

<template>
  <div class="character-edit-page">
    <!-- 顶部导航栏 -->
    <div class="top-bar row items-center q-px-md q-py-sm bg-white">
      <q-btn flat round icon="arrow_back" @click="router.push('/pet-config')">
        <q-tooltip>返回</q-tooltip>
      </q-btn>
      <div class="text-h6 q-ml-sm text-weight-medium">
        角色配置：{{ model?.name || "加载中..." }}
      </div>
      <q-space />
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-center q-py-xl">
      <q-spinner color="primary" size="40px" />
      <div class="q-mt-sm text-grey-6">加载角色数据...</div>
    </div>

    <template v-else>
      <!-- Tabs -->
      <q-tabs
        v-model="tab"
        class="bg-white q-px-md"
        active-color="primary"
        indicator-color="primary"
        align="left"
        narrow-indicator
      >
        <q-tab name="basic" icon="settings" label="基本信息" />
        <q-tab name="expressions" icon="mood" label="表情立绘" />
        <q-tab name="llm" icon="smart_toy" label="LLM 提示词" />
      </q-tabs>

      <q-separator />

      <q-tab-panels v-model="tab" class="bg-grey-2" animated>
        <!-- ════════════════════════════════════════
             TAB 1: 基本信息（不变）
             ════════════════════════════════════════ -->
        <q-tab-panel name="basic">
          <div class="row justify-center">
            <div class="col-12 col-md-8 col-lg-6">
              <q-card flat bordered>
                <q-card-section>
                  <div class="text-h6 q-mb-md">基本设置</div>
                  <q-input v-model="basicForm.name" label="角色名称 *" outlined dense class="q-mb-md" />
                  <q-input v-model="basicForm.model_start_path" label="引擎启动路径" outlined dense class="q-mb-md" placeholder="引擎目录或 api_v2.py 路径">
                    <template v-slot:after>
                      <q-btn icon="folder_open" color="primary" flat dense @click="
                        open({ multiple: false, directory: true, filters: undefined }).then(r => { if (r) basicForm.model_start_path = r })
                      " />
                    </template>
                  </q-input>
                  <q-input v-model="basicForm.api_base_url" label="API 地址" outlined dense class="q-mb-md" placeholder="http://127.0.0.1:9880" />
                  <q-input v-model="basicForm.gpt_model" label="GPT 模型路径" outlined dense class="q-mb-md">
                    <template v-slot:after>
                      <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('gpt_model')" />
                    </template>
                  </q-input>
                  <q-input v-model="basicForm.sovits_model" label="SoVITS 模型路径" outlined dense class="q-mb-md">
                    <template v-slot:after>
                      <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('sovits_model')" />
                    </template>
                  </q-input>
                  <q-input v-model="basicForm.config_path" label="配置文件路径" outlined dense class="q-mb-md">
                    <template v-slot:after>
                      <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('config_path')" />
                    </template>
                  </q-input>
                  <q-btn unelevated color="primary" :loading="savingBasic" icon="save" label="保存基本设置" class="full-width" @click="saveBasic" />
                </q-card-section>
              </q-card>
            </div>
          </div>
        </q-tab-panel>

        <!-- ════════════════════════════════════════
             TAB 2: 表情立绘（固定 5 种表情）
             ════════════════════════════════════════ -->
        <q-tab-panel name="expressions">
          <div class="row justify-center">
            <div class="col-12 col-md-10 col-lg-8">
              <div class="text-subtitle1 text-weight-medium text-grey-8 q-mb-md">
                每种表情可配置对应的参考音频、TTS 提示文本和立绘图片
              </div>

              <div class="row q-col-gutter-md">
                <div
                  v-for="meta in EXPR_META"
                  :key="meta.name"
                  class="col-12 col-sm-6"
                >
                  <q-card
                    flat
                    bordered
                    class="expr-card cursor-pointer"
                    @click="pickIllustration(meta.name)"
                  >
                    <q-card-section class="q-pb-sm">
                      <div class="row items-center no-wrap">
                        <!-- 立绘缩略图 -->
                        <div class="expr-thumb">
                          <q-icon
                            v-if="!getExpr(meta.name)?.illustration_path"
                            :name="meta.icon"
                            size="32px"
                            color="primary"
                          />
                          <img
                            v-else
                            :src="'http://127.0.0.1:9880/' + getExpr(meta.name)!.illustration_path"
                            class="expr-thumb-img"
                            @error="($event.target as HTMLImageElement).style.display = 'none'"
                          />
                        </div>
                        <div class="q-ml-sm flex-1">
                          <div class="text-weight-medium text-body1">
                            {{ meta.display_name }}
                          </div>
                          <div class="text-caption text-grey-6 q-mt-xs">
                            音频：{{ getExpr(meta.name)?.ref_audio_path ? getExpr(meta.name)!.ref_audio_path.split(/[/\\]/).pop() : "未设置" }}
                          </div>
                          <div class="text-caption text-grey-5 ellipsis" :title="getExpr(meta.name)?.illustration_path">
                            立绘：{{ getExpr(meta.name)?.illustration_path || "未设置" }}
                          </div>
                        </div>
                        <q-btn flat round dense icon="more_vert" size="sm" color="grey-5"
                          @click.stop="openEditExpr(meta.name)">
                          <q-tooltip>编辑音频 / 提示词</q-tooltip>
                        </q-btn>
                      </div>
                    </q-card-section>
                  </q-card>
                </div>
              </div>
            </div>
          </div>
        </q-tab-panel>

        <!-- ════════════════════════════════════════
             TAB 3: LLM 提示词配置（不变）
             ════════════════════════════════════════ -->
        <q-tab-panel name="llm">
          <div class="row justify-center">
            <div class="col-12 col-md-10 col-lg-8">
              <q-card flat bordered class="q-mb-md">
                <q-card-section>
                  <div class="text-h6 q-mb-md">提示词设置</div>
                  <div class="q-mb-md">
                    <div class="text-caption text-grey-7 q-mb-xs">System Prompt（系统提示词）</div>
                    <q-input v-model="llmConfig.system_prompt" outlined type="textarea" :rows="6"
                      placeholder="设定角色的身份、性格、说话方式等...&#10;例如：你是{character_name}，一个温柔体贴的虚拟助手..." />
                  </div>
                  <div class="q-mb-md">
                    <div class="text-caption text-grey-7 q-mb-xs">User Prompt 模板</div>
                    <q-input v-model="llmConfig.user_prompt_template" outlined type="textarea" :rows="3"
                      placeholder="用户消息的模板格式&#10;可用变量: {user_message}, {character_name}, {history}" />
                    <div class="text-caption text-grey-5 q-mt-xs">
                      可用变量：<code>{user_message}</code> 用户输入，<code>{character_name}</code> 角色名，<code>{history}</code> 对话历史
                    </div>
                  </div>
                  <div class="q-mb-md">
                    <div class="text-caption text-grey-7 q-mb-xs">开场白</div>
                    <q-input v-model="llmConfig.greeting_message" outlined type="textarea" :rows="2"
                      placeholder="角色首次见面时说的话（可选）" />
                  </div>
                </q-card-section>
              </q-card>

              <q-card flat bordered class="q-mb-md">
                <q-card-section>
                  <div class="text-h6 q-mb-md">生成参数</div>
                  <div class="row q-col-gutter-lg">
                    <div class="col-12 col-sm-6">
                      <div class="text-caption text-grey-7 q-mb-xs">温度 (Temperature): {{ llmConfig.temperature }}</div>
                      <q-slider v-model="llmConfig.temperature" :min="0" :max="2" :step="0.05" color="primary" label label-always />
                      <div class="text-caption text-grey-5">越低越确定，越高越有创意</div>
                    </div>
                    <div class="col-12 col-sm-6">
                      <div class="text-caption text-grey-7 q-mb-xs">Top-P: {{ llmConfig.top_p }}</div>
                      <q-slider v-model="llmConfig.top_p" :min="0" :max="1" :step="0.05" color="primary" label label-always />
                      <div class="text-caption text-grey-5">核采样阈值</div>
                    </div>
                    <div class="col-12 col-sm-6">
                      <div class="text-caption text-grey-7 q-mb-xs">最大 Token: {{ llmConfig.max_tokens }}</div>
                      <q-slider v-model="llmConfig.max_tokens" :min="64" :max="4096" :step="64" color="primary" label label-always />
                    </div>
                    <div class="col-12 col-sm-6">
                      <div class="text-caption text-grey-7 q-mb-xs">存在惩罚 (Presence Penalty): {{ llmConfig.presence_penalty }}</div>
                      <q-slider v-model="llmConfig.presence_penalty" :min="-2" :max="2" :step="0.1" color="primary" label label-always />
                    </div>
                    <div class="col-12 col-sm-6">
                      <div class="text-caption text-grey-7 q-mb-xs">频率惩罚 (Frequency Penalty): {{ llmConfig.frequency_penalty }}</div>
                      <q-slider v-model="llmConfig.frequency_penalty" :min="-2" :max="2" :step="0.1" color="primary" label label-always />
                    </div>
                  </div>
                </q-card-section>
              </q-card>

              <q-btn unelevated color="primary" size="lg" :loading="savingLlm" icon="save" label="保存 LLM 配置" class="full-width" @click="saveLlm" />
            </div>
          </div>
        </q-tab-panel>
      </q-tab-panels>
    </template>

    <!-- ── 差分编辑对话框 ── -->
    <q-dialog v-model="exprDialog" persistent>
      <q-card style="min-width: 520px; max-width: 640px">
        <q-card-section class="q-py-md bg-primary text-white">
          <div class="text-h6">
            编辑表情：{{ editingMeta?.display_name || "" }}
          </div>
        </q-card-section>

        <q-scroll-area class="q-pa-md" style="max-height: 70vh">
          <!-- 立绘 -->
          <div class="text-caption text-grey-7 q-mb-xs">立绘图片</div>
          <div class="row items-center q-col-gutter-sm q-mb-md">
            <div class="col">
              <q-input v-model="exprForm.illustration_path" outlined dense placeholder="图片路径（png/jpg/webp）" />
            </div>
            <div class="col-auto">
              <q-btn icon="image" color="primary" flat dense @click="pickFile('illustration_path')" />
            </div>
          </div>

          <q-separator class="q-mb-md" />

          <!-- 参考音频 -->
          <div class="text-caption text-grey-7 q-mb-xs">参考音频</div>
          <div class="row items-center q-col-gutter-sm q-mb-md">
            <div class="col">
              <q-input v-model="exprForm.ref_audio_path" outlined dense placeholder="引擎内的音频路径" />
            </div>
            <div class="col-auto">
              <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('ref_audio_path')" />
            </div>
          </div>

          <!-- 提示文本 -->
          <q-input v-model="exprForm.prompt_text" label="提示文本 (prompt_text)" outlined dense class="q-mb-md" type="textarea" :rows="2" />

          <!-- 语言 -->
          <div class="row q-col-gutter-md q-mb-md">
            <div class="col-6">
              <q-select v-model="exprForm.prompt_lang" :options="LANG_OPTIONS" label="提示语言" outlined dense />
            </div>
            <div class="col-6">
              <q-select v-model="exprForm.text_lang" :options="LANG_OPTIONS" label="目标语言" outlined dense />
            </div>
          </div>
        </q-scroll-area>

        <q-card-actions align="right" class="q-pa-md">
          <q-btn flat label="取消" v-close-popup color="grey-7" />
          <q-btn label="保存" color="primary" unelevated :loading="savingExpr" @click="saveExpr" />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </div>
</template>

<style scoped>
.character-edit-page {
  min-height: 100%;
}

.top-bar {
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  position: sticky;
  top: 0;
  z-index: 10;
}

.expr-card {
  border-radius: 10px;
  transition: transform 0.15s, box-shadow 0.15s;
}

.expr-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.expr-thumb {
  width: 56px;
  height: 56px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(var(--q-primary), 0.06);
  flex-shrink: 0;
  overflow: hidden;
}

.expr-thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.flex-1 {
  flex: 1;
  min-width: 0;
}

.ellipsis {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

:deep(.q-tab-panels) {
  background: #f5f5f5;
}
</style>
