import { invoke } from "@tauri-apps/api/core"
import type { CharacterModel } from "@/types"

/** 创建角色模型 */
export function createModel(model: Omit<CharacterModel, "id" | "created_at" | "updated_at">) {
  return invoke<number>("create_model", { model })
}

/** 查询所有角色模型 */
export function listModels() {
  return invoke<CharacterModel[]>("list_models")
}

/** 根据 ID 查询单个角色模型 */
export function getModel(id: number) {
  return invoke<CharacterModel | null>("get_model", { id })
}

/** 更新角色模型 */
export function updateModel(id: number, model: Omit<CharacterModel, "id" | "created_at" | "updated_at">) {
  return invoke<boolean>("update_model", { id, model })
}

/** 删除角色模型 */
export function deleteModel(id: number) {
  return invoke<boolean>("delete_model", { id })
}
