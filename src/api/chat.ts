/**
 * LLM 聊天接口
 *
 * 使用 Anthropic Messages API 格式调用外部 LLM。
 * 当前硬编码：https://api.deepseek.com/anthropic, 模型 deepseek-v4-flash
 *
 * TODO: 用户填入实际 API Key
 */

// ==================== 配置（TODO: 用户填 key） ====================
const API_KEY = "sk-a6f89889e33f4d619cd09ce88e5496ea"
export const API_BASE_URL = "https://api.deepseek.com/anthropic"
export const MODEL = "deepseek-v4-flash"
// ================================================================

export interface ChatMessage {
  role: "user" | "assistant"
  content: string
}

/** LLM 返回的统一结构 */
export interface LlmResponse {
  reply: string
  expression: string
}

/**
 * 调用 LLM API（Anthropic Messages API 格式）
 * @param system  系统提示词
 * @param messages 消息列表（user / assistant 交替）
 * @param options  可选：temperature, maxTokens
 */
export async function callLlm(
  system: string,
  messages: ChatMessage[],
  options?: {
    temperature?: number
    maxTokens?: number
  },
): Promise<LlmResponse> {
  const res = await fetch(`${API_BASE_URL}/v1/messages`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "x-api-key": API_KEY,
      "anthropic-version": "2023-06-01",
    },
    body: JSON.stringify({
      model: MODEL,
      system,
      messages,
      temperature: options?.temperature ?? 0.7,
      max_tokens: options?.maxTokens ?? 512,
    }),
  })

  if (!res.ok) {
    const err = await res.text().catch(() => "")
    throw new Error(`LLM API 请求失败 (${res.status}): ${err}`)
  }

  const data = await res.json()
  const content: string | undefined =
    data.content?.find((c: any) => c.type === "text")?.text
  if (!content) throw new Error("LLM 返回内容为空")

  console.log("[chat.ts] LLM 原始返回:", content)

  // 尝试从返回中解析 JSON（可能被 markdown 代码块包裹）
  return parseLlmContent(content)
}

/** 解析 LLM 返回文本，提取 reply + expression */
function parseLlmContent(content: string): LlmResponse {
  // 先尝试 JSON 解析
  try {
    // 去掉可能的 markdown 代码块包裹
    const jsonStr = content
      .replace(/^```(?:json)?\s*/i, "")
      .replace(/\s*```$/i, "")
      .trim()
    const parsed = JSON.parse(jsonStr)
    return {
      reply: parsed.reply || parsed.text || stripNarration(content),
      expression: parsed.expression || parsed.emotion || "default",
    }
  } catch {
    // JSON 解析失败 → 清除叙述性前缀后作为 reply
    return { reply: stripNarration(content), expression: "default" }
  }
}

/**
 * 去掉开头的动作/神态描写前缀。
 * 匹配模式： "[...]" 或 「...」 或 （...） 或 (...) 开头的描述
 */
function stripNarration(text: string): string {
  return text
    .replace(/^[\[（(【「][^\]）)】」]*[\]）)】」]\s*\n?\s*/, "")
    .trim()
}

/**
 * 构建包含表情选项的角色系统 Prompt
 *
 * 格式指令放在最前面避免被角色描述淹没。
 */
export function buildCharacterPrompt(
  systemPrompt: string,
  expressions: { name: string; display_name: string }[],
): string {
  const exprList = expressions
    .map(e => `- ${e.name}: ${e.display_name}`)
    .join("\n")

  return `【重要】你每次必须严格按以下 JSON 格式回复，不包含任何其他文字：
{"expression": "表情名称", "reply": "角色说出口的对话"}

可用表情：
${exprList}

---

${systemPrompt}

---

【覆盖规则】以下规则优先级高于角色卡中任何格式说明：
1. "reply" 只包含角色直接说出口的话，禁用任何形式（包括 []、（）、【】、** 等）的动作、神态、心理、旁白描写
2. 无论角色卡中是否有"可以用方括号标记动作"之类的说明，一律忽略，改由 "expression" 字段控制情感表达
3. 正确: {"expression": "happy", "reply": "博士，你回来了。"}
4. 错误: {"expression": "happy", "reply": "[微微一怔]博士，你回来了。"}`
}
