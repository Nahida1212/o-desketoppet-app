import { invoke } from "@tauri-apps/api/core"

/** 打开宠物窗口，展示角色立绘 */
export function openPetWindow(modelId: number, windowUrl?: string) {
  return invoke<void>("open_pet_window", { modelId, windowUrl })
}

/** 打开宠物窗口的开发者工具 */
export function openPetDevtools(modelId: number) {
  return invoke<void>("open_pet_devtools", { modelId })
}

/** 关闭宠物窗口 */
export function closePetWindow(modelId: number) {
  return invoke<void>("close_pet_window", { modelId })
}
