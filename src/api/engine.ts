import { invoke } from "@tauri-apps/api/core"

/** 启动 GPT-SoVITS 进程 */
export function startModel(modelId: number) {
  return invoke<string>("start_model", { modelId })
}

/** 停止 GPT-SoVITS 进程 */
export function stopModel(modelId: number) {
  return invoke<string>("stop_model", { modelId })
}

/** 查询模型进程状态 */
export function getModelStatus(modelId: number) {
  return invoke<boolean>("get_model_status", { modelId })
}
