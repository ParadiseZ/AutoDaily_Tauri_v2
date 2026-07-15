use chrono::Utc;
pub(crate) use domain_script::ScriptTransferRecord;
use infra_sqlite::{
    CreateScriptTransferRecordInput, FinishScriptTransferRecordInput,
    clear_script_transfer_records, delete_script_transfer_record as delete_record,
    finish_script_transfer_record as finish_record, insert_script_transfer_record as insert_record,
    list_script_transfer_records,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;

const SCRIPT_TRANSFER_EVENT: &str = "script-transfer";
const TRANSFER_STATE_RUNNING: u8 = 0;
const TRANSFER_STATE_PAUSED: u8 = 1;
const TRANSFER_STATE_DELETE_REQUESTED: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptTransferControlState {
    Running,
    Paused,
    DeleteRequested,
}

#[derive(Debug)]
pub struct ScriptTransferControl {
    state: AtomicU8,
    notify: Notify,
}

impl ScriptTransferControl {
    fn new() -> Self {
        Self {
            state: AtomicU8::new(TRANSFER_STATE_RUNNING),
            notify: Notify::new(),
        }
    }

    pub fn pause(&self) {
        if self.state() == ScriptTransferControlState::DeleteRequested {
            return;
        }
        self.state.store(TRANSFER_STATE_PAUSED, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        if self.state() == ScriptTransferControlState::DeleteRequested {
            return;
        }
        self.state.store(TRANSFER_STATE_RUNNING, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn request_delete(&self) {
        self.state
            .store(TRANSFER_STATE_DELETE_REQUESTED, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn state(&self) -> ScriptTransferControlState {
        match self.state.load(Ordering::SeqCst) {
            TRANSFER_STATE_PAUSED => ScriptTransferControlState::Paused,
            TRANSFER_STATE_DELETE_REQUESTED => ScriptTransferControlState::DeleteRequested,
            _ => ScriptTransferControlState::Running,
        }
    }

    pub async fn wait_for_signal(&self) {
        self.notify.notified().await;
    }
}

fn transfer_controls() -> &'static Mutex<HashMap<String, Arc<ScriptTransferControl>>> {
    static CONTROLS: OnceLock<Mutex<HashMap<String, Arc<ScriptTransferControl>>>> = OnceLock::new();
    CONTROLS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register_script_transfer_control(record_id: &str) -> Arc<ScriptTransferControl> {
    let control = Arc::new(ScriptTransferControl::new());
    let mut controls = transfer_controls()
        .lock()
        .expect("script transfer control mutex poisoned");
    controls.insert(record_id.to_string(), control.clone());
    control
}

pub fn get_script_transfer_control(record_id: &str) -> Option<Arc<ScriptTransferControl>> {
    let controls = transfer_controls()
        .lock()
        .expect("script transfer control mutex poisoned");
    controls.get(record_id).cloned()
}

pub fn unregister_script_transfer_control(record_id: &str) {
    let mut controls = transfer_controls()
        .lock()
        .expect("script transfer control mutex poisoned");
    controls.remove(record_id);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTransferProgressEvent {
    pub id: String,
    pub direction: String,
    pub local_script_id: Option<String>,
    pub cloud_script_id: Option<String>,
    pub script_name: Option<String>,
    pub status: String,
    pub model_file_count: i64,
    pub completed_model_file_count: i64,
    pub current_file_name: Option<String>,
    pub latest_file_name: Option<String>,
    pub bytes_transferred: i64,
    pub total_bytes: i64,
    pub latest_message: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub updated_at: String,
}

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn emit_script_transfer_event(app_handle: &AppHandle, payload: &ScriptTransferProgressEvent) {
    let _ = app_handle.emit(SCRIPT_TRANSFER_EVENT, payload);
}

pub async fn insert_script_transfer_record(
    input: CreateScriptTransferRecordInput,
) -> Result<(), String> {
    let updated_at = input.finished_at.clone().unwrap_or_else(now_rfc3339);
    insert_record(input, updated_at).await
}

pub async fn finish_script_transfer_record(
    input: FinishScriptTransferRecordInput,
) -> Result<(), String> {
    let updated_at = input.finished_at.clone().unwrap_or_else(now_rfc3339);
    finish_record(input, updated_at).await
}

#[tauri::command]
pub async fn list_script_transfer_records_cmd(
    direction: Option<String>,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<ScriptTransferRecord>, String> {
    list_script_transfer_records(direction, local_script_id, cloud_script_id, limit).await
}

#[tauri::command]
pub async fn delete_script_transfer_record_cmd(record_id: String) -> Result<(), String> {
    if let Some(control) = get_script_transfer_control(&record_id) {
        control.request_delete();
    }
    delete_record(&record_id).await
}

#[tauri::command]
pub async fn clear_script_transfer_records_cmd(
    direction: Option<String>,
    local_script_id: Option<String>,
    cloud_script_id: Option<String>,
) -> Result<(), String> {
    clear_script_transfer_records(direction, local_script_id, cloud_script_id).await
}

#[tauri::command]
pub async fn pause_script_transfer_record_cmd(record_id: String) -> Result<(), String> {
    let Some(control) = get_script_transfer_control(&record_id) else {
        return Err("传输已结束，无法暂停".to_string());
    };
    control.pause();
    Ok(())
}

#[tauri::command]
pub async fn resume_script_transfer_record_cmd(record_id: String) -> Result<(), String> {
    let Some(control) = get_script_transfer_control(&record_id) else {
        return Err("传输已结束，无法继续".to_string());
    };
    control.resume();
    Ok(())
}
