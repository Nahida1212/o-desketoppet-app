/** 角色差分 / 表情配置 */
export interface CharacterExpression {
  id?: number | null
  model_id: number
  /** 表情 key，如 "default", "happy", "sad" */
  name: string
  /** 显示名称，如 "常态", "开心", "难过" */
  display_name: string
  ref_audio_path: string
  prompt_text: string
  prompt_lang: string
  text_lang: string
  /** 立绘图片路径 */
  illustration_path: string
  sort_order: number
  created_at?: string
  updated_at?: string
}
