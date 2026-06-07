/** 宠物状态 */
export interface PetState {
  id: string
  name: string
  mood: "happy" | "normal" | "sad"
  energy: number
  hunger: number
}

/** 角色模型 */
export interface CharacterModel {
  id?: number | null
  name: string
  /** 模型启动文件地址 */
  model_start_path: string
  ref_audio_path: string
  prompt_text: string
  prompt_lang: string
  text_lang: string
  gpt_model: string
  sovits_model: string
  api_base_url: string
  config_path: string
  created_at?: string
  updated_at?: string
}

/** 通用 API 响应 */
export interface ApiResponse<T = unknown> {
  code: number
  message: string
  data: T
}
