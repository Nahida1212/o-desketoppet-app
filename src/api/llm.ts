import { invoke } from "@tauri-apps/api/core"
import type { CharacterLlmConfig } from "@/types"

/** 获取 LLM 提示词配置 */
export function getLlmConfig(modelId: number) {
  return invoke<CharacterLlmConfig | null>("get_llm_config", { modelId })
}

/** 保存 LLM 提示词配置 */
export function saveLlmConfig(modelId: number, config: Omit<CharacterLlmConfig, "id" | "created_at" | "updated_at">) {
  return invoke<number>("save_llm_config", { modelId, config })
}

/** 确保指定模型的 5 个默认表情差分存在（用于旧模型迁移） */
export function ensureDefaultExpressions(modelId: number) {
  return invoke<void>("ensure_default_expressions", { modelId })
}
