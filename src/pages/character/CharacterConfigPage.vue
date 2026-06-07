<script setup lang="ts">
import { ref, onMounted } from "vue"
import { useQuasar } from "quasar"
import { open } from "@tauri-apps/plugin-dialog"
import { listModels, createModel, updateModel, deleteModel } from "@/api/model"
import type { CharacterModel } from "@/types"

const $q = useQuasar()

// ── 数据列表 ──
const models = ref<CharacterModel[]>([])
const loading = ref(false)

async function fetchList() {
  loading.value = true
  try {
    models.value = await listModels()
  } catch (e) {
    $q.notify({ type: "negative", message: `加载失败: ${e}` })
  } finally {
    loading.value = false
  }
}

// ── 表单对话框 ──
const showDialog = ref(false)
const editingId = ref<number | null>(null)
const form = ref({
  name: "",
  model_start_path: "",
  ref_audio_path: "",
  prompt_text: "",
  prompt_lang: "zh",
  text_lang: "zh",
  gpt_model: "",
  sovits_model: "",
  api_base_url: "",
  config_path: "",
})
const saving = ref(false)

function openCreate() {
  editingId.value = null
  form.value = {
    name: "",
    model_start_path: "",
    ref_audio_path: "",
    prompt_text: "",
    prompt_lang: "zh",
    text_lang: "zh",
    gpt_model: "",
    sovits_model: "",
    api_base_url: "",
    config_path: "",
  }
  showDialog.value = true
}

async function openEdit(model: CharacterModel) {
  editingId.value = model.id!
  form.value = {
    name: model.name,
    model_start_path: model.model_start_path,
    ref_audio_path: model.ref_audio_path,
    prompt_text: model.prompt_text,
    prompt_lang: model.prompt_lang,
    text_lang: model.text_lang,
    gpt_model: model.gpt_model,
    sovits_model: model.sovits_model,
    api_base_url: model.api_base_url,
    config_path: model.config_path,
  }
  showDialog.value = true
}

async function save() {
  if (!form.value.name.trim()) {
    $q.notify({ type: "warning", message: "请输入模型名称" })
    return
  }
  saving.value = true
  try {
    if (editingId.value != null) {
      await updateModel(editingId.value, form.value)
      $q.notify({ type: "positive", message: "更新成功" })
    } else {
      await createModel(form.value)
      $q.notify({ type: "positive", message: "创建成功" })
    }
    showDialog.value = false
    await fetchList()
  } catch (e) {
    $q.notify({ type: "negative", message: `保存失败: ${e}` })
  } finally {
    saving.value = false
  }
}

async function handleDelete(id: number) {
  $q.dialog({
    title: "确认删除",
    message: "确定要删除该角色模型吗？此操作不可恢复。",
    cancel: true,
    persistent: true,
  }).onOk(async () => {
    try {
      await deleteModel(id)
      $q.notify({ type: "positive", message: "删除成功" })
      await fetchList()
    } catch (e) {
      $q.notify({ type: "negative", message: `删除失败: ${e}` })
    }
  })
}

/** 打开文件选择器，选中后填入对应字段 */
async function pickFile(field: string) {
  const filters: Record<string, { name: string; extensions: string[] }[]> = {
    ref_audio_path: [{ name: "音频文件", extensions: ["wav", "mp3", "flac", "ogg", "m4a", "aac"] }],
    gpt_model: [{ name: "GPT 模型", extensions: ["pth", "pt", "ckpt"] }],
    sovits_model: [{ name: "SoVITS 模型", extensions: ["pth", "pt", "ckpt"] }],
    config_path: [{ name: "配置文件", extensions: ["json", "yaml", "yml", "toml"] }],
  }
  const selected = await open({
    multiple: false,
    directory: false,
    filters: filters[field] ?? [{ name: "所有文件", extensions: ["*"] }],
  })
  if (selected) {
    ;(form.value as Record<string, string>)[field] = selected
  }
}

// ── 初始化 ──
onMounted(fetchList)
</script>

<template>
  <div class="q-pa-lg">
    <!-- 页面标题 -->
    <div class="row items-center justify-between q-mb-lg">
      <div>
        <div class="text-h5 text-weight-medium q-mb-xs">
          <q-icon name="smart_toy" color="primary" class="q-mr-sm" />
          角色模型配置
        </div>
        <q-breadcrumbs class="text-grey-6 q-mt-sm">
          <q-breadcrumbs-el label="角色模型配置" />
        </q-breadcrumbs>
      </div>
      <q-btn
        label="新建模型"
        color="primary"
        icon="add"
        unelevated
        @click="openCreate"
      />
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-center q-py-xl">
      <q-spinner color="primary" size="40px" />
      <div class="q-mt-sm text-grey-6">加载中...</div>
    </div>

    <!-- 模型列表 -->
    <div v-else-if="models.length > 0" class="row q-col-gutter-md">
      <div
        v-for="(m, idx) in models"
        :key="m.id ?? idx"
        class="col-12"
      >
        <q-card flat bordered>
          <q-card-section>
            <div class="row items-center justify-between">
              <div class="text-h6 text-weight-medium">{{ m.name }}</div>
              <div>
                <q-btn flat round icon="edit" color="primary" size="sm" @click="openEdit(m)">
                  <q-tooltip>编辑</q-tooltip>
                </q-btn>
                <q-btn flat round icon="delete" color="negative" size="sm" @click="m.id != null && handleDelete(m.id)">
                  <q-tooltip>删除</q-tooltip>
                </q-btn>
              </div>
            </div>

            <q-separator class="q-my-sm" />

            <div class="row q-col-gutter-md">
              <div class="col-12 col-sm-6">
                <div class="text-caption text-grey-7">模型启动文件</div>
                <div class="text-body2 text-weight-medium">{{ m.model_start_path || "-" }}</div>
              </div>
              <div class="col-12 col-sm-6">
                <div class="text-caption text-grey-7">API 地址</div>
                <div class="text-body2 text-weight-medium">{{ m.api_base_url || "-" }}</div>
              </div>
              <div class="col-12 col-sm-6">
                <div class="text-caption text-grey-7">GPT 模型</div>
                <div class="text-body2">{{ m.gpt_model || "-" }}</div>
              </div>
              <div class="col-12 col-sm-6">
                <div class="text-caption text-grey-7">SoVITS 模型</div>
                <div class="text-body2">{{ m.sovits_model || "-" }}</div>
              </div>
              <div class="col-12 col-sm-6">
                <div class="text-caption text-grey-7">提示语言</div>
                <div class="text-body2">{{ m.prompt_lang }}</div>
              </div>
              <div class="col-12 col-sm-6">
                <div class="text-caption text-grey-7">目标语言</div>
                <div class="text-body2">{{ m.text_lang }}</div>
              </div>
            </div>

            <div class="q-mt-sm text-caption text-grey-5">
              创建于 {{ m.created_at }}
            </div>
          </q-card-section>
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
      <div class="text-body2 q-mt-sm q-mb-md">创建一个角色模型来开始语音合成吧！</div>
      <q-btn label="新建模型" color="primary" icon="add" unelevated @click="openCreate" />
    </div>
  </div>

  <!-- 新建/编辑对话框 -->
  <q-dialog v-model="showDialog" persistent maximized>
    <q-card class="column" style="max-width: 640px; max-height: 90vh;">
      <q-card-section class="q-py-md bg-primary text-white">
        <div class="text-h6">
          {{ editingId != null ? "编辑模型" : "新建模型" }}
        </div>
      </q-card-section>

      <q-scroll-area class="col-grow q-pa-md">
        <q-input
          v-model="form.name"
          label="模型名称 *"
          outlined
          dense
          class="q-mb-md"
          :rules="[v => !!v.trim() || '请输入名称']"
        />
        <q-input
          v-model="form.model_start_path"
          label="模型启动文件地址"
          outlined
          dense
          class="q-mb-md"
          placeholder="如: D:/models/my-model/start.bat"
        >
          <template v-slot:after>
            <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('model_start_path')" />
          </template>
        </q-input>
        <q-input
          v-model="form.ref_audio_path"
          label="参考音频路径 (ref_audio_path)"
          outlined
          dense
          class="q-mb-md"
        >
          <template v-slot:after>
            <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('ref_audio_path')" />
          </template>
        </q-input>
        <q-input
          v-model="form.prompt_text"
          label="提示文本 (prompt_text)"
          outlined
          dense
          class="q-mb-md"
          type="textarea"
          rows="2"
        />
        <div class="row q-col-gutter-md q-mb-md">
          <div class="col-6">
            <q-select
              v-model="form.prompt_lang"
              label="提示语言 (prompt_lang)"
              :options="[
                { label: '中文', value: 'zh' },
                { label: '英文', value: 'en' },
                { label: '日文', value: 'ja' },
              ]"
              emit-value
              map-options
              outlined
              dense
            />
          </div>
          <div class="col-6">
            <q-select
              v-model="form.text_lang"
              label="目标语言 (text_lang)"
              :options="[
                { label: '中文', value: 'zh' },
                { label: '英文', value: 'en' },
                { label: '日文', value: 'ja' },
              ]"
              emit-value
              map-options
              outlined
              dense
            />
          </div>
        </div>
        <q-input
          v-model="form.gpt_model"
          label="GPT 模型路径"
          outlined
          dense
          class="q-mb-md"
        >
          <template v-slot:after>
            <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('gpt_model')" />
          </template>
        </q-input>
        <q-input
          v-model="form.sovits_model"
          label="SoVITS 模型路径"
          outlined
          dense
          class="q-mb-md"
        >
          <template v-slot:after>
            <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('sovits_model')" />
          </template>
        </q-input>
        <q-input
          v-model="form.api_base_url"
          label="API 地址 (api_base_url)"
          outlined
          dense
          class="q-mb-md"
          placeholder="如: http://127.0.0.1:9880"
        />
        <q-input
          v-model="form.config_path"
          label="配置文件路径 (config_path)"
          outlined
          dense
          class="q-mb-md"
        >
          <template v-slot:after>
            <q-btn icon="folder_open" color="primary" flat dense @click="pickFile('config_path')" />
          </template>
        </q-input>
      </q-scroll-area>

      <q-card-actions align="right" class="q-pa-md">
        <q-btn flat label="取消" v-close-popup color="grey-7" />
        <q-btn
          :label="editingId != null ? '保存' : '创建'"
          color="primary"
          unelevated
          :loading="saving"
          @click="save"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
