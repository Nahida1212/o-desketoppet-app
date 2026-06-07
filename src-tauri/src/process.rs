use std::collections::HashMap;
use std::process::Child;
use std::sync::Mutex;

/// 追踪已启动的模型进程：model_id → Child 句柄
pub struct ProcessManager {
    pub processes: Mutex<HashMap<i64, Child>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        if let Ok(mut map) = self.processes.lock() {
            for (_, mut child) in map.drain() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
