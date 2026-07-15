use sqlx::SqlitePool;
use tokio::sync::OnceCell;

mod bootstrap;
mod devices;
mod migrations;
mod schedules;
mod schema;
mod scripts;

pub(crate) static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

pub use bootstrap::{get_pool, init_db_and_migrate_with_path, init_db_with_path};
pub use devices::{delete_device_with_assignments, get_all_devices, get_device, save_device};
pub use schedules::{
    cleanup_expired_schedule_records, fail_active_assignment_schedules_by_device,
    has_complete_assignment_schedule_batch, insert_assignment_schedule,
    insert_assignment_schedule_batch, load_assignment_schedules_by_device,
    load_next_planned_assignment_schedule, reactivate_retryable_planner_schedules_for_device,
    stop_active_assignment_schedules_by_device, stop_planned_planner_schedules_by_device,
    sync_active_planner_schedule_order_indices, sync_active_planner_schedules_from_queue,
    update_assignment_schedule_status, update_assignment_schedule_status_by_dispatch_id,
};
pub use schedules::{
    clear_schedules_by_device, clear_schedules_by_script, compact_assignment_indices,
    delete_assignment, delete_template_value, delete_time_template, find_latest_success_schedule,
    find_template_value, find_template_value_exact, get_time_template,
    has_active_assignment_schedules, insert_execution_schedule, list_assigned_device_ids_by_script,
    list_assigned_device_ids_by_time_template, list_assignments, list_execution_schedules,
    list_time_templates, reorder_assignment_indices, save_assignment, save_template_value,
    save_time_template, upsert_template_value,
};
pub use scripts::{
    CreateScriptTransferRecordInput, FinishScriptTransferRecordInput,
    clear_script_transfer_records, delete_script_transfer_record, finish_script_transfer_record,
    insert_script_transfer_record, list_script_transfer_records,
};
pub use scripts::{
    batch_insert_script_related, delete_script_graph_in_transaction, save_cloned_script_graph,
    save_script_editor_graph,
};
pub use scripts::{
    delete_policy, delete_policy_group, delete_policy_set, list_group_ids_in_set,
    list_group_policy_links, list_policies, list_policy_groups, list_policy_ids_in_group,
    list_policy_sets, list_script_tasks, list_set_group_links, replace_group_policy_links,
    replace_set_group_links, save_policy, save_policy_group, save_policy_set,
};
pub use scripts::{
    delete_script, ensure_existing_script_editable, ensure_stored_script_editable,
    find_dev_script_by_cloud_id, get_script, list_scripts, save_script,
};
