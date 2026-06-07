use std::path::Path;
use tauri::{AppHandle, State};

use crate::db::{self, DbState};
use crate::logging::log_msg;
use crate::process::ProcessManager;

/// 从 api_base_url 解析 host 和 port（默认 0.0.0.0:9880）
fn parse_host_port(url: &str, default_host: &str, default_port: &str) -> (String, String) {
    let url = url.trim().trim_start_matches("http://").trim_start_matches("https://");
    if let Some((host, port)) = url.split_once(':') {
        (host.to_string(), port.to_string())
    } else if let Some(host) = url.split_once('/').map(|(h, _)| h).or_else(|| {
        if !url.is_empty() { Some(url) } else { None }
    }) {
        (host.to_string(), default_port.to_string())
    } else {
        (default_host.to_string(), default_port.to_string())
    }
}

/// 启动 GPT-SoVITS API 进程
#[tauri::command]
pub fn start_model(
    app: AppHandle,
    state: State<DbState>,
    proc_mgr: State<ProcessManager>,
    model_id: i64,
) -> Result<String, String> {
    log_msg(&app, "INFO", &format!("尝试启动模型 #{}", model_id));

    // 1. 从数据库获取模型
    let model = {
        let conn = state.conn.lock().map_err(|e| {
            let err = format!("数据库锁失败: {}", e);
            log_msg(&app, "ERROR", &err);
            err
        })?;
        let m = db::get_model(&conn, model_id)
            .map_err(|e| {
                let err = format!("数据库查询失败: {}", e);
                log_msg(&app, "ERROR", &err);
                err
            })?
            .ok_or_else(|| {
                let err = format!("模型 #{} 不存在", model_id);
                log_msg(&app, "ERROR", &err);
                err
            })?;
        log_msg(&app, "INFO", &format!("已加载模型「{}」", m.name));
        m
    };

    // 2. 解析引擎路径 — 支持目录路径或 api_v2.py 文件路径
    let start_path = Path::new(&model.model_start_path);
    log_msg(&app, "INFO", &format!("model_start_path = {}", model.model_start_path));

    let (engine_dir, script_path) = if start_path.is_dir() {
        log_msg(&app, "INFO", "→ 检测为目录路径，从中查找 api_v2.py");
        (start_path.to_path_buf(), start_path.join("api_v2.py"))
    } else if start_path.is_file() {
        log_msg(&app, "INFO", "→ 检测为文件路径，使用其所在目录作为引擎目录");
        let parent = start_path.parent().ok_or_else(|| {
            let err = format!("无法获取父目录: {}", model.model_start_path);
            log_msg(&app, "ERROR", &err);
            err
        })?;
        (parent.to_path_buf(), start_path.to_path_buf())
    } else {
        let err = format!("路径不存在或无法访问: {}", model.model_start_path);
        log_msg(&app, "ERROR", &err);
        return Err(err);
    };

    log_msg(&app, "INFO", &format!("引擎目录 = {}", engine_dir.display()));
    log_msg(&app, "INFO", &format!("脚本路径 = {}", script_path.display()));

    if !script_path.exists() {
        let err = format!("api_v2.py 不存在: {}", script_path.display());
        log_msg(&app, "ERROR", &err);
        return Err(err);
    }

    // 3. 查找 Python 运行时
    let python_rel = if cfg!(target_os = "windows") {
        "runtime/python.exe"
    } else {
        "runtime/bin/python3"
    };
    let python_path = engine_dir.join(python_rel);
    log_msg(&app, "INFO", &format!("Python 路径 = {}", python_path.display()));

    if !python_path.exists() {
        let err = format!("Python 运行时不存在: {}\n请检查 model_start_path 是否指向 GPT-SoVITS 引擎目录（包含 runtime/ 和 api_v2.py）", python_path.display());
        log_msg(&app, "ERROR", &err);
        return Err(err);
    }

    // 4. 解析 host / port
    let (host, port) = parse_host_port(&model.api_base_url, "0.0.0.0", "9880");
    log_msg(&app, "INFO", &format!("API 地址 = {}:{}", host, port));

    // 5. 构造命令
    let mut cmd = std::process::Command::new(&python_path);
    cmd.current_dir(&engine_dir)
        .arg(&script_path)
        .arg("-a").arg(&host)
        .arg("-p").arg(&port)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env("PYTHONUNBUFFERED", "1");

    // 配置文件
    if !model.config_path.is_empty() {
        cmd.arg("-c").arg(&model.config_path);
        log_msg(&app, "INFO", &format!("配置文件 = {}", model.config_path));
    }

    log_msg(&app, "INFO", &format!("完整命令: {:?}", cmd));

    // 6. 启动进程
    match cmd.spawn() {
        Ok(mut child) => {
            let pid = child.id();
            log_msg(&app, "INFO", &format!("进程启动成功 (PID: {})", pid));

            // 异步读取 stdout
            let app_stdout = app.clone();
            if let Some(stdout) = child.stdout.take() {
                std::thread::spawn(move || {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        match line {
                            Ok(text) => log_msg(&app_stdout, "STDOUT", &text),
                            Err(_) => break,
                        }
                    }
                    log_msg(&app_stdout, "INFO", "进程 stdout 已关闭");
                });
            }

            // 异步读取 stderr
            let app_stderr = app.clone();
            if let Some(stderr) = child.stderr.take() {
                std::thread::spawn(move || {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        match line {
                            Ok(text) => log_msg(&app_stderr, "STDERR", &text),
                            Err(_) => break,
                        }
                    }
                    log_msg(&app_stderr, "INFO", "进程 stderr 已关闭");
                });
            }

            // 记录进程到 ProcessManager
            let mut processes = proc_mgr.processes.lock().map_err(|e| {
                let err = format!("ProcessManager 锁失败: {}", e);
                log_msg(&app, "ERROR", &err);
                err
            })?;

            if let Some(mut old_child) = processes.remove(&model_id) {
                log_msg(&app, "WARN", &format!("终止旧进程 (PID: {})", old_child.id()));
                let _ = old_child.kill();
                let _ = old_child.wait();
            }

            processes.insert(model_id, child);
            drop(processes);

            Ok(format!("{} 启动成功 (PID: {})", model.name, pid))
        }
        Err(e) => {
            let err = format!("进程启动失败: {}\n  命令: {:?}\n  详细: 请确认 Python 运行时完整且所有路径存在", e, cmd);
            log_msg(&app, "ERROR", &err);
            Err(err)
        }
    }
}

/// 停止模型进程
#[tauri::command]
pub fn stop_model(
    proc_mgr: State<ProcessManager>,
    model_id: i64,
) -> Result<String, String> {
    let child = {
        let mut processes = proc_mgr.processes.lock().map_err(|e| e.to_string())?;
        processes.remove(&model_id)
    };

    match child {
        Some(mut child) => {
            let pid = child.id();
            child.kill().map_err(|e| format!("终止进程失败: {}", e))?;
            let _ = child.wait();
            Ok(format!("进程已终止 (PID: {})", pid))
        }
        None => Err("该模型没有正在运行的进程".to_string()),
    }
}

/// 查询模型运行状态（通过 try_wait 检测真实进程状态）
#[tauri::command]
pub fn get_model_status(
    proc_mgr: State<ProcessManager>,
    model_id: i64,
) -> Result<bool, String> {
    let mut processes = proc_mgr.processes.lock().map_err(|e| e.to_string())?;
    match processes.get_mut(&model_id) {
        Some(child) => match child.try_wait() {
            Ok(Some(_)) => {
                processes.remove(&model_id);
                Ok(false)
            }
            Ok(None) => Ok(true),
            Err(_) => {
                processes.remove(&model_id);
                Ok(false)
            }
        },
        None => Ok(false),
    }
}
