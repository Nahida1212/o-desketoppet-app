use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Manager;

/// TTS 推理配置文件顶层结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsInferConfig {
    pub custom: VersionConfig,
    pub v1: VersionConfig,
    pub v2: VersionConfig,
    #[serde(rename = "v2Pro")]
    pub v2_pro: VersionConfig,
    #[serde(rename = "v2ProPlus")]
    pub v2_pro_plus: VersionConfig,
    pub v3: VersionConfig,
    pub v4: VersionConfig,
}

/// 单个版本的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionConfig {
    pub bert_base_path: String,
    pub cnhuhbert_base_path: String,
    pub device: String,
    pub is_half: bool,
    pub t2s_weights_path: String,
    pub version: String,
    pub vits_weights_path: String,
}

/// 各版本的预设模型路径
const BASE_BERT: &str = "GPT_SoVITS/pretrained_models/chinese-roberta-wwm-ext-large";
const BASE_HUBERT: &str = "GPT_SoVITS/pretrained_models/chinese-hubert-base";

fn preset(version: &str, t2s: &str, vits: &str, device: &str, is_half: bool) -> VersionConfig {
    VersionConfig {
        bert_base_path: BASE_BERT.into(),
        cnhuhbert_base_path: BASE_HUBERT.into(),
        device: device.into(),
        is_half,
        t2s_weights_path: t2s.into(),
        version: version.into(),
        vits_weights_path: vits.into(),
    }
}

impl Default for TtsInferConfig {
    fn default() -> Self {
        Self {
            custom: VersionConfig {
                bert_base_path: BASE_BERT.into(),
                cnhuhbert_base_path: BASE_HUBERT.into(),
                device: "cuda".into(),
                is_half: true,
                t2s_weights_path: String::new(),
                version: "v2Pro".into(),
                vits_weights_path: String::new(),
            },
            v1: preset("v1",
                "GPT_SoVITS/pretrained_models/s1bert25hz-2kh-longer-epoch=68e-step=50232.ckpt",
                "GPT_SoVITS/pretrained_models/s2G488k.pth",
                "cpu", false),
            v2: preset("v2",
                "GPT_SoVITS/pretrained_models/gsv-v2final-pretrained/s1bert25hz-5kh-longer-epoch=12-step=369668.ckpt",
                "GPT_SoVITS/pretrained_models/gsv-v2final-pretrained/s2G2333k.pth",
                "cpu", false),
            v2_pro: preset("v2Pro",
                "GPT_SoVITS/pretrained_models/s1v3.ckpt",
                "GPT_SoVITS/pretrained_models/v2Pro/s2Gv2Pro.pth",
                "cpu", false),
            v2_pro_plus: preset("v2ProPlus",
                "GPT_SoVITS/pretrained_models/s1v3.ckpt",
                "GPT_SoVITS/pretrained_models/v2Pro/s2Gv2ProPlus.pth",
                "cpu", false),
            v3: preset("v3",
                "GPT_SoVITS/pretrained_models/s1v3.ckpt",
                "GPT_SoVITS/pretrained_models/s2Gv3.pth",
                "cpu", false),
            v4: preset("v4",
                "GPT_SoVITS/pretrained_models/s1v3.ckpt",
                "GPT_SoVITS/pretrained_models/gsv-v4-pretrained/s2Gv4.pth",
                "cpu", false),
        }
    }
}

/// 根据角色模型生成 TTS 配置文件，写入 configs 目录
/// 返回生成的 YAML 文件路径
pub fn generate_model_config(
    model_name: &str,
    model_id: i64,
    gpt_model: &str,
    sovits_model: &str,
    _prompt_text: &str,
    _prompt_lang: &str,
    _text_lang: &str,
    _ref_audio_path: &str,
    configs_dir: &Path,
) -> Result<PathBuf, String> {
    // 确保 configs 目录存在
    std::fs::create_dir_all(configs_dir).map_err(|e| format!("创建 configs 目录失败: {}", e))?;

    let mut config = TtsInferConfig::default();

    // 填入用户模型的路径
    config.custom.t2s_weights_path = gpt_model.to_string();
    config.custom.vits_weights_path = sovits_model.to_string();

    // 如果 ref_audio_path 不为空，尝试将其转为绝对路径后填入
    // （推理时参考音频路径通常在 YAML 之外单独指定，这里保留字段以待扩展）

    // 生成安全的文件名
    let safe_name: String = model_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect();
    let filename = format!("{}_{}.yaml", model_id, safe_name);
    let filepath = configs_dir.join(&filename);

    // 序列化为 YAML
    let yaml_str =
        serde_yaml::to_string(&config).map_err(|e| format!("YAML 序列化失败: {}", e))?;

    // 写入文件
    std::fs::write(&filepath, &yaml_str).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(filepath)
}

/// 删除模型对应的配置文件
pub fn delete_config_file(config_path: &str) -> Result<(), String> {
    if !config_path.is_empty() && Path::new(config_path).exists() {
        std::fs::remove_file(config_path)
            .map_err(|e| format!("删除配置文件失败: {}", e))?;
    }
    Ok(())
}

/// 获取 configs 目录路径
pub fn get_configs_dir(app: &tauri::AppHandle) -> PathBuf {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data dir");
    app_dir.join("configs")
}
