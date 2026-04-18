use crate::infrastructure::core::{DeviceId, HashMap, ScriptId, SessionId};
use crate::infrastructure::ipc::message::{
    RunTarget, RuntimeExecutionPolicy, RuntimeQueueItem, RuntimeSessionSnapshot, ScriptBundleSnapshot,
};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ChildRuntimeSession {
    snapshot: RuntimeSessionSnapshot,
    bundles_by_script: HashMap<ScriptId, ScriptBundleSnapshot>,
}

#[derive(Debug, Clone)]
pub struct ChildRuntimeSessionSummary {
    pub session_id: SessionId,
    pub device_id: DeviceId,
    pub run_target: RunTarget,
    pub queue_len: usize,
}

type SharedChildRuntimeSession = Arc<RwLock<Option<ChildRuntimeSession>>>;

static RUNTIME_SESSION: OnceLock<SharedChildRuntimeSession> = OnceLock::new();

impl ChildRuntimeSession {
    pub fn new(snapshot: RuntimeSessionSnapshot) -> Self {
        let bundles_by_script = snapshot
            .script_bundles
            .iter()
            .cloned()
            .map(|bundle| (bundle.script_id, bundle))
            .collect();

        Self { snapshot, bundles_by_script }
    }

    pub fn summary(&self) -> ChildRuntimeSessionSummary {
        ChildRuntimeSessionSummary {
            session_id: self.snapshot.session_id,
            device_id: self.snapshot.device_id,
            run_target: self.snapshot.run_target.clone(),
            queue_len: self.snapshot.queue.len(),
        }
    }

    pub fn bundle(&self, script_id: ScriptId) -> Option<ScriptBundleSnapshot> {
        self.bundles_by_script.get(&script_id).cloned()
    }

    pub fn queue_item(
        &self,
        assignment_id: crate::infrastructure::core::ScheduleId,
    ) -> Option<RuntimeQueueItem> {
        self.snapshot
            .queue
            .iter()
            .find(|item| item.assignment_id == assignment_id)
            .cloned()
    }

    pub fn runtime_policy(&self) -> RuntimeExecutionPolicy {
        self.snapshot.runtime_policy.clone()
    }
}

pub fn get_runtime_session_store() -> SharedChildRuntimeSession {
    RUNTIME_SESSION
        .get_or_init(|| Arc::new(RwLock::new(None)))
        .clone()
}

pub async fn replace_runtime_session(
    snapshot: RuntimeSessionSnapshot,
) -> ChildRuntimeSessionSummary {
    let session = ChildRuntimeSession::new(snapshot);
    let summary = session.summary();
    let store = get_runtime_session_store();
    *store.write().await = Some(session);
    summary
}

pub async fn clear_runtime_session() -> Option<ChildRuntimeSessionSummary> {
    let store = get_runtime_session_store();
    let mut guard = store.write().await;
    let summary = guard.as_ref().map(ChildRuntimeSession::summary);
    *guard = None;
    summary
}

pub async fn get_script_bundle_snapshot(script_id: ScriptId) -> Option<ScriptBundleSnapshot> {
    let store = get_runtime_session_store();
    let guard = store.read().await;
    guard.as_ref().and_then(|session| session.bundle(script_id))
}

pub async fn get_runtime_queue_item(
    assignment_id: crate::infrastructure::core::ScheduleId,
) -> Option<RuntimeQueueItem> {
    let store = get_runtime_session_store();
    let guard = store.read().await;
    guard
        .as_ref()
        .and_then(|session| session.queue_item(assignment_id))
}

pub async fn get_runtime_execution_policy() -> Option<RuntimeExecutionPolicy> {
    let store = get_runtime_session_store();
    let guard = store.read().await;
    guard.as_ref().map(ChildRuntimeSession::runtime_policy)
}

pub fn try_current_session_summary() -> Option<ChildRuntimeSessionSummary> {
    let store = get_runtime_session_store();
    let guard = store.try_read().ok()?;
    guard.as_ref().map(ChildRuntimeSession::summary)
}
