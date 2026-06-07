/** LLM 角色提示词配置 */
export interface CharacterLlmConfig {
  id?: number | null
  model_id: number
  system_prompt: string
  user_prompt_template: string
  greeting_message: string
  temperature: number
  max_tokens: number
  top_p: number
  presence_penalty: number
  frequency_penalty: number
  created_at?: string
  updated_at?: string
}
