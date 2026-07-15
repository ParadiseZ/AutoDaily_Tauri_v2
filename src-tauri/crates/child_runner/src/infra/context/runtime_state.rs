use ad_kernel::ids::{PolicyGroupId, PolicyId, PolicySetId};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct PolicyState {
    pub skip_flag: bool,
    pub done_flag: bool,
    pub exec_cur: u32,
    pub click_pos: Option<u16>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ActionState {
    pub exec_cur: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskState {
    pub enabled_flag: bool,
    pub skip_flag: bool,
    pub done_flag: bool,
    pub exec_cur: u32,
}

impl Default for TaskState {
    fn default() -> Self {
        Self {
            enabled_flag: true,
            skip_flag: false,
            done_flag: false,
            exec_cur: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicySetBindingSource {
    PolicySet(PolicySetId),
    PolicyGroup(PolicyGroupId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PolicySetBindingOp {
    pub source: PolicySetBindingSource,
    pub top: bool,
    pub reverse: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicyGroupBindingSource {
    Policy(PolicyId),
    PolicyGroup(PolicyGroupId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PolicyGroupBindingOp {
    pub source: PolicyGroupBindingSource,
    pub top: bool,
    pub reverse: bool,
}

#[cfg(test)]
mod tests {
    use super::{ActionState, PolicyState, TaskState};

    #[test]
    fn defaults_keep_a_task_enabled_and_all_counters_clear() {
        assert!(TaskState::default().enabled_flag);
        assert_eq!(TaskState::default().exec_cur, 0);
        assert_eq!(PolicyState::default().click_pos, None);
        assert_eq!(ActionState::default().exec_cur, 0);
    }
}
