/** 通用工具函数 */

/**
 * 格式化日期
 */
export function formatDate(date: Date, format = "YYYY-MM-DD"): string {
  const map: Record<string, string | number> = {
    YYYY: date.getFullYear(),
    MM: String(date.getMonth() + 1).padStart(2, "0"),
    DD: String(date.getDate()).padStart(2, "0"),
    HH: String(date.getHours()).padStart(2, "0"),
    mm: String(date.getMinutes()).padStart(2, "0"),
    ss: String(date.getSeconds()).padStart(2, "0"),
  }
  let result = format
  for (const [key, val] of Object.entries(map)) {
    result = result.replace(key, String(val))
  }
  return result
}

/**
 * 延迟等待
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * 随机整数 [min, max]
 */
export function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}
