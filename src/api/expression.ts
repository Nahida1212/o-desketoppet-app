import { invoke } from "@tauri-apps/api/core"
import type { CharacterExpression } from "@/types"

/** 查询角色所有差分 */
export function listExpressions(modelId: number) {
  return invoke<CharacterExpression[]>("list_expressions", { modelId })
}

/** 创建差分 */
export function createExpression(expression: Omit<CharacterExpression, "id" | "created_at" | "updated_at">) {
  return invoke<number>("create_expression", { expression })
}

/** 更新差分 */
export function updateExpression(id: number, expression: Omit<CharacterExpression, "id" | "created_at" | "updated_at">) {
  return invoke<boolean>("update_expression", { id, expression })
}

/** 删除差分 */
export function deleteExpression(id: number) {
  return invoke<boolean>("delete_expression", { id })
}
