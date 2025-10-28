use chrono::{DateTime, Local, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;
use crate::domain::entities::scripts::script_info::{ScriptId, ScriptPriority};

/// 任务队列中的任务项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// 脚本ID
    pub script_id: ScriptId,
    /// 计划执行时间
    pub scheduled_time: DateTime<Local>,
    /// 优先级
    pub priority: ScriptPriority,
    /// 预估执行时间（毫秒）
    pub estimated_duration_ms: u64,
    /// 任务创建时间
    pub created_at: DateTime<Local>,
    /// 重试次数
    pub retry_count: u32,
    /// 最大重试次数
    pub max_retries: u32,
}

impl ScheduledTask {
    pub fn new(
        script_id: ScriptId,
        scheduled_time: DateTime<Local>,
        priority: ScriptPriority,
        estimated_duration_ms: u64,
        max_retries: u32,
    ) -> Self {
        Self {
            script_id,
            scheduled_time,
            priority,
            estimated_duration_ms,
            created_at: Local::now(),
            retry_count: 0,
            max_retries,
        }
    }

    /// 检查任务是否应该执行
    pub fn should_execute(&self, current_time: DateTime<Local>) -> bool {
        current_time >= self.scheduled_time
    }

    /// 检查任务是否可以重试
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// 增加重试次数
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

impl Eq for ScheduledTask {}

impl PartialEq for ScheduledTask {
    fn eq(&self, other: &Self) -> bool {
        self.scheduled_time == other.scheduled_time 
            && self.priority == other.priority
            && self.script_id == other.script_id
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 首先按计划执行时间排序（越早越优先）
        let time_cmp = other.scheduled_time.cmp(&self.scheduled_time);
        if time_cmp != Ordering::Equal {
            return time_cmp;
        }

        // 时间相同时按优先级排序（优先级越高越优先）
        let priority_cmp = self.priority.cmp(&other.priority);
        if priority_cmp != Ordering::Equal {
            return priority_cmp;
        }

        // 优先级相同时按创建时间排序（越早创建越优先）
        other.created_at.cmp(&self.created_at)
    }
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// 任务队列状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskQueueStatus {
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 停止
    Stopped,
}

/// 任务队列统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueueStats {
    /// 待执行任务数量
    pub pending_tasks: usize,
    /// 正在执行的任务数量
    pub running_tasks: usize,
    /// 今日已完成任务数量
    pub completed_today: u64,
    /// 今日失败任务数量
    pub failed_today: u64,
    /// 平均任务执行时间（毫秒）
    pub average_execution_time_ms: u64,
    /// 队列负载（0-100%）
    pub queue_load_percent: u8,
}

impl Default for TaskQueueStats {
    fn default() -> Self {
        Self {
            pending_tasks: 0,
            running_tasks: 0,
            completed_today: 0,
            failed_today: 0,
            average_execution_time_ms: 0,
            queue_load_percent: 0,
        }
    }
}

/// 脚本任务队列
#[derive(Debug)]
pub struct TaskQueue {
    /// 优先级队列（堆）
    task_heap: BinaryHeap<ScheduledTask>,
    /// 正在执行的任务
    running_tasks: HashMap<ScriptId, ScheduledTask>,
    /// 失败任务队列（用于重试）
    failed_tasks: VecDeque<ScheduledTask>,
    /// 队列状态
    status: TaskQueueStatus,
    /// 最大并发任务数
    max_concurrent_tasks: usize,
    /// 统计信息
    stats: TaskQueueStats,
}

impl TaskQueue {
    /// 创建新的任务队列
    pub fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            task_heap: BinaryHeap::new(),
            running_tasks: HashMap::new(),
            failed_tasks: VecDeque::new(),
            status: TaskQueueStatus::Stopped,
            max_concurrent_tasks,
            stats: TaskQueueStats::default(),
        }
    }

    /// 添加任务到队列
    pub fn enqueue_task(&mut self, task: ScheduledTask) {
        tracing::info!("添加任务到队列: script_id={}, scheduled_time={}", 
                      task.script_id, task.scheduled_time);
        self.task_heap.push(task);
        self.update_stats();
    }

    /// 获取下一个应该执行的任务
    pub fn get_next_task(&mut self, current_time: DateTime<Local>) -> Option<ScheduledTask> {
        if self.status != TaskQueueStatus::Running {
            return None;
        }

        // 检查是否已达到最大并发数
        if self.running_tasks.len() >= self.max_concurrent_tasks {
            return None;
        }

        // 首先检查失败任务队列中的重试任务
        while let Some(mut failed_task) = self.failed_tasks.pop_front() {
            if failed_task.should_execute(current_time) && failed_task.can_retry() {
                failed_task.increment_retry();
                tracing::info!("重试失败任务: script_id={}, retry_count={}", 
                              failed_task.script_id, failed_task.retry_count);
                return Some(failed_task);
            }
            
            // 如果不能重试或时间未到，放回队列末尾
            if failed_task.can_retry() {
                self.failed_tasks.push_back(failed_task);
                break;
            }
            // 超过最大重试次数的任务直接丢弃
        }

        // 从主队列获取任务
        while let Some(task) = self.task_heap.peek() {
            if !task.should_execute(current_time) {
                // 还没到执行时间
                break;
            }

            let task = self.task_heap.pop().unwrap();
            
            // 检查脚本是否已在运行
            if self.running_tasks.contains_key(&task.script_id) {
                tracing::warn!("脚本已在运行，跳过任务: script_id={}", task.script_id);
                continue;
            }

            return Some(task);
        }

        None
    }

    /// 标记任务开始执行
    pub fn mark_task_running(&mut self, task: ScheduledTask) {
        tracing::info!("任务开始执行: script_id={}", task.script_id);
        self.running_tasks.insert(task.script_id.clone(), task);
        self.update_stats();
    }

    /// 标记任务完成
    pub fn mark_task_completed(&mut self, script_id: &ScriptId, success: bool) {
        if let Some(task) = self.running_tasks.remove(script_id) {
            if success {
                tracing::info!("任务执行成功: script_id={}", script_id);
                self.stats.completed_today += 1;
            } else {
                tracing::warn!("任务执行失败: script_id={}", script_id);
                self.stats.failed_today += 1;
                
                // 将失败任务加入重试队列
                if task.can_retry() {
                    self.failed_tasks.push_back(task);
                }
            }
        }
        self.update_stats();
    }

    /// 取消指定脚本的所有任务
    pub fn cancel_script_tasks(&mut self, script_id: &ScriptId) {
        // 从主队列中移除
        let mut remaining_tasks = Vec::new();
        while let Some(task) = self.task_heap.pop() {
            if task.script_id != *script_id {
                remaining_tasks.push(task);
            }
        }
        
        // 重建堆
        for task in remaining_tasks {
            self.task_heap.push(task);
        }

        // 从失败队列中移除
        self.failed_tasks.retain(|task| task.script_id != *script_id);

        // 从运行任务中移除（这应该通过外部停止进程来处理）
        self.running_tasks.remove(script_id);
        
        tracing::info!("已取消脚本的所有任务: script_id={}", script_id);
        self.update_stats();
    }

    /// 清空队列
    pub fn clear(&mut self) {
        self.task_heap.clear();
        self.failed_tasks.clear();
        self.running_tasks.clear();
        self.update_stats();
        tracing::info!("任务队列已清空");
    }

    /// 启动队列
    pub fn start(&mut self) {
        self.status = TaskQueueStatus::Running;
        tracing::info!("任务队列已启动");
    }

    /// 暂停队列
    pub fn pause(&mut self) {
        self.status = TaskQueueStatus::Paused;
        tracing::info!("任务队列已暂停");
    }

    /// 停止队列
    pub fn stop(&mut self) {
        self.status = TaskQueueStatus::Stopped;
        tracing::info!("任务队列已停止");
    }

    /// 获取队列状态
    pub fn get_status(&self) -> TaskQueueStatus {
        self.status.clone()
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &TaskQueueStats {
        &self.stats
    }

    /// 获取指定脚本的待执行任务数量
    pub fn get_pending_tasks_count(&self, script_id: &ScriptId) -> usize {
        let heap_count = self.task_heap.iter()
            .filter(|task| task.script_id == *script_id)
            .count();
        
        let failed_count = self.failed_tasks.iter()
            .filter(|task| task.script_id == *script_id)
            .count();

        heap_count + failed_count
    }

    /// 检查脚本是否正在运行
    pub fn is_script_running(&self, script_id: &ScriptId) -> bool {
        self.running_tasks.contains_key(script_id)
    }

    /// 获取所有正在运行的脚本ID
    pub fn get_running_script_ids(&self) -> Vec<ScriptId> {
        self.running_tasks.keys().cloned().collect()
    }

    /// 更新统计信息
    fn update_stats(&mut self) {
        self.stats.pending_tasks = self.task_heap.len() + self.failed_tasks.len();
        self.stats.running_tasks = self.running_tasks.len();
        
        // 计算队列负载
        let total_capacity = self.max_concurrent_tasks;
        let current_load = self.running_tasks.len();
        self.stats.queue_load_percent = if total_capacity > 0 {
            ((current_load as f64 / total_capacity as f64) * 100.0) as u8
        } else {
            0
        };
    }

    /// 获取队列概览信息
    pub fn get_queue_overview(&self) -> String {
        format!(
            "TaskQueue[状态:{:?}, 待执行:{}, 运行中:{}, 负载:{}%]",
            self.status,
            self.stats.pending_tasks,
            self.stats.running_tasks,
            self.stats.queue_load_percent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_task_queue_basic_operations() {
        let mut queue = TaskQueue::new(2);
        let now = Local::now();
        
        // 测试添加任务
        let task1 = ScheduledTask::new(
            "script1".to_string(),
            now + Duration::minutes(1),
            ScriptPriority::High,
            5000,
            3,
        );
        
        queue.enqueue_task(task1);
        assert_eq!(queue.get_stats().pending_tasks, 1);
        
        // 测试获取任务（时间未到）
        assert!(queue.get_next_task(now).is_none());
        
        // 测试获取任务（时间已到）
        queue.start();
        let future_time = now + Duration::minutes(2);
        let next_task = queue.get_next_task(future_time);
        assert!(next_task.is_some());
    }

    #[test]
    fn test_task_priority_ordering() {
        let mut queue = TaskQueue::new(10);
        let now = Local::now();
        
        queue.start();
        
        // 添加不同优先级的任务
        let task_low = ScheduledTask::new(
            "script_low".to_string(),
            now,
            ScriptPriority::Low,
            1000,
            3,
        );
        
        let task_high = ScheduledTask::new(
            "script_high".to_string(),
            now,
            ScriptPriority::High,
            1000,
            3,
        );
        
        queue.enqueue_task(task_low);
        queue.enqueue_task(task_high);
        
        // 高优先级任务应该先被取出
        let next_task = queue.get_next_task(now).unwrap();
        assert_eq!(next_task.script_id, "script_high");
        assert_eq!(next_task.priority, ScriptPriority::High);
    }

    #[test]
    fn test_concurrent_task_limit() {
        let mut queue = TaskQueue::new(1); // 最大并发数为1
        let now = Local::now();
        
        queue.start();
        
        let task1 = ScheduledTask::new("script1".to_string(), now, ScriptPriority::Normal, 1000, 3);
        let task2 = ScheduledTask::new("script2".to_string(), now, ScriptPriority::Normal, 1000, 3);
        
        queue.enqueue_task(task1.clone());
        queue.enqueue_task(task2);
        
        // 获取第一个任务并标记为运行中
        let first_task = queue.get_next_task(now).unwrap();
        queue.mark_task_running(first_task);
        
        // 应该无法获取第二个任务（已达到并发限制）
        assert!(queue.get_next_task(now).is_none());
        
        // 完成第一个任务后应该能获取第二个任务
        queue.mark_task_completed(&"script1".to_string(), true);
        assert!(queue.get_next_task(now).is_some());
    }
}
