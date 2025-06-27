// WezTerm Multi-Process Development Framework - Task Tracker
// Provides time tracking, progress monitoring, and productivity analytics

use super::types::{TaskId};
use super::{current_timestamp, format_duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Task tracking and time management system
#[derive(Debug)]
pub struct TaskTracker {
    /// Active time tracking sessions
    active_sessions: RwLock<HashMap<TaskId, TrackingSession>>,
    
    /// Completed tracking sessions
    completed_sessions: RwLock<Vec<CompletedSession>>,
    
    /// Task productivity metrics
    productivity_metrics: RwLock<HashMap<TaskId, ProductivityMetrics>>,
    
    /// Daily summaries
    daily_summaries: RwLock<HashMap<String, DailySummary>>, // date -> summary
    
    /// Tracker statistics
    stats: RwLock<TrackerStats>,
}

impl TaskTracker {
    /// Create a new task tracker
    pub fn new() -> Self {
        Self {
            active_sessions: RwLock::new(HashMap::new()),
            completed_sessions: RwLock::new(Vec::new()),
            productivity_metrics: RwLock::new(HashMap::new()),
            daily_summaries: RwLock::new(HashMap::new()),
            stats: RwLock::new(TrackerStats::new()),
        }
    }

    /// Start tracking a task
    pub async fn start_task(&self, task_id: &TaskId) {
        let session = TrackingSession {
            task_id: task_id.clone(),
            started_at: current_timestamp(),
            last_activity: current_timestamp(),
            total_active_time: 0,
            break_time: 0,
            interruptions: 0,
            is_paused: false,
            segments: Vec::new(),
        };

        {
            let mut active = self.active_sessions.write().await;
            active.insert(task_id.clone(), session);
        }

        {
            let mut stats = self.stats.write().await;
            stats.sessions_started += 1;
        }

        info!("Started tracking task: {}", task_id);
    }

    /// Stop tracking a task
    pub async fn stop_task(&self, task_id: &TaskId) -> Option<Duration> {
        let session = {
            let mut active = self.active_sessions.write().await;
            active.remove(task_id)
        };

        if let Some(mut session) = session {
            let now = current_timestamp();
            session.total_active_time += now - session.last_activity;

            // Create completed session
            let productivity_score = self.calculate_productivity_score(&session);
            let completed = CompletedSession {
                task_id: task_id.clone(),
                started_at: session.started_at,
                ended_at: now,
                total_duration: now - session.started_at,
                active_duration: session.total_active_time,
                break_duration: session.break_time,
                interruptions: session.interruptions,
                segments: session.segments,
                productivity_score,
            };

            let duration = Duration::from_secs(completed.active_duration);

            // Store completed session
            {
                let mut completed_sessions = self.completed_sessions.write().await;
                completed_sessions.push(completed.clone());
            }

            // Update daily summary
            self.update_daily_summary(&completed).await;

            // Update productivity metrics
            self.update_productivity_metrics(task_id, &completed).await;

            // Update statistics
            {
                let mut stats = self.stats.write().await;
                stats.sessions_completed += 1;
                stats.total_tracked_time += completed.active_duration;
            }

            info!("Stopped tracking task: {} (duration: {})", task_id, format_duration(duration));
            Some(duration)
        } else {
            None
        }
    }

    /// Pause tracking for a task
    pub async fn pause_task(&self, task_id: &TaskId) -> bool {
        let mut active = self.active_sessions.write().await;
        if let Some(session) = active.get_mut(task_id) {
            if !session.is_paused {
                let now = current_timestamp();
                session.total_active_time += now - session.last_activity;
                session.is_paused = true;
                
                // Add segment
                session.segments.push(TimeSegment {
                    started_at: session.last_activity,
                    ended_at: now,
                    segment_type: SegmentType::Active,
                });

                debug!("Paused tracking for task: {}", task_id);
                return true;
            }
        }
        false
    }

    /// Resume tracking for a task
    pub async fn resume_task(&self, task_id: &TaskId) -> bool {
        let mut active = self.active_sessions.write().await;
        if let Some(session) = active.get_mut(task_id) {
            if session.is_paused {
                let now = current_timestamp();
                session.break_time += now - session.last_activity;
                session.last_activity = now;
                session.is_paused = false;

                debug!("Resumed tracking for task: {}", task_id);
                return true;
            }
        }
        false
    }

    /// Record an interruption for a task
    pub async fn record_interruption(&self, task_id: &TaskId, duration_seconds: u64) {
        let mut active = self.active_sessions.write().await;
        if let Some(session) = active.get_mut(task_id) {
            session.interruptions += 1;
            session.break_time += duration_seconds;
            
            // Add interruption segment
            let now = current_timestamp();
            session.segments.push(TimeSegment {
                started_at: now - duration_seconds,
                ended_at: now,
                segment_type: SegmentType::Interruption,
            });

            debug!("Recorded interruption for task: {} ({}s)", task_id, duration_seconds);
        }
    }

    /// Update activity for a task (heartbeat)
    pub async fn update_activity(&self, task_id: &TaskId) {
        let mut active = self.active_sessions.write().await;
        if let Some(session) = active.get_mut(task_id) {
            if !session.is_paused {
                session.last_activity = current_timestamp();
            }
        }
    }

    /// Get current tracking session for a task
    pub async fn get_active_session(&self, task_id: &TaskId) -> Option<TrackingSession> {
        let active = self.active_sessions.read().await;
        active.get(task_id).cloned()
    }

    /// Get all active tracking sessions
    pub async fn get_active_sessions(&self) -> Vec<TrackingSession> {
        let active = self.active_sessions.read().await;
        active.values().cloned().collect()
    }

    /// Get task history for a specific task
    pub async fn get_task_history(&self, task_id: &TaskId) -> Vec<CompletedSession> {
        let completed = self.completed_sessions.read().await;
        completed.iter()
            .filter(|session| session.task_id == *task_id)
            .cloned()
            .collect()
    }

    /// Get productivity metrics for a task
    pub async fn get_productivity_metrics(&self, task_id: &TaskId) -> Option<ProductivityMetrics> {
        let metrics = self.productivity_metrics.read().await;
        metrics.get(task_id).cloned()
    }

    /// Get daily summary for a specific date
    pub async fn get_daily_summary(&self, date: &str) -> Option<DailySummary> {
        let summaries = self.daily_summaries.read().await;
        summaries.get(date).cloned()
    }

    /// Get tracker statistics
    pub async fn get_stats(&self) -> TrackerStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Generate enhanced productivity report
    pub async fn generate_enhanced_productivity_report(&self, since_timestamp: Option<u64>) -> ProductivityReport {
        let completed = self.completed_sessions.read().await;
        let active = self.active_sessions.read().await;
        let since = since_timestamp.unwrap_or(0);
        
        // Filter sessions by timestamp
        let relevant_completed: Vec<_> = completed
            .iter()
            .filter(|s| s.started_at >= since)
            .collect();
        
        let relevant_active: Vec<_> = active
            .values()
            .filter(|s| s.started_at >= since)
            .collect();
        
        // Calculate metrics
        let total_sessions = relevant_completed.len() + relevant_active.len();
        let total_time = relevant_completed.iter().map(|s| s.total_duration).sum::<u64>()
            + relevant_active.iter().map(|s| s.get_current_duration()).sum::<u64>();
        
        let total_focused_time = relevant_completed.iter().map(|s| s.active_duration).sum::<u64>()
            + relevant_active.iter().map(|s| s.get_focused_time()).sum::<u64>();
        
        let total_break_time = relevant_completed.iter().map(|s| s.break_duration).sum::<u64>()
            + relevant_active.iter().map(|s| s.break_time).sum::<u64>();
        
        let total_interruptions = relevant_completed.iter().map(|s| s.interruptions).sum::<u32>()
            + relevant_active.iter().map(|s| s.interruptions).sum::<u32>();
        
        // Calculate averages
        let avg_session_duration = if total_sessions > 0 {
            total_time / total_sessions as u64
        } else {
            0
        };
        
        let focus_efficiency = if total_time > 0 {
            (total_focused_time as f64 / total_time as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_interruptions_per_session = if total_sessions > 0 {
            total_interruptions as f64 / total_sessions as f64
        } else {
            0.0
        };
        
        // Daily breakdown
        let mut daily_stats = std::collections::HashMap::new();
        for session in &relevant_completed {
            let day = session.started_at / 86400; // Convert to days
            let entry = daily_stats.entry(day).or_insert(DailyStats {
                date: day * 86400,
                total_time: 0,
                focused_time: 0,
                sessions: 0,
                interruptions: 0,
            });
            entry.total_time += session.total_duration;
            entry.focused_time += session.active_duration;
            entry.sessions += 1;
            entry.interruptions += session.interruptions as u64;
        }
        
        let daily_breakdown: Vec<_> = daily_stats.into_values().collect();
        
        ProductivityReport {
            period_start: since,
            period_end: current_timestamp(),
            total_sessions: total_sessions as u64,
            total_time,
            total_focused_time,
            total_break_time,
            total_interruptions: total_interruptions as u64,
            avg_session_duration,
            focus_efficiency,
            avg_interruptions_per_session,
            daily_breakdown,
        }
    }

    /// Get task-specific productivity insights
    pub async fn get_task_insights(&self, task_id: &TaskId) -> Option<TaskInsights> {
        let completed = self.completed_sessions.read().await;
        let active = self.active_sessions.read().await;
        
        let task_sessions: Vec<_> = completed
            .iter()
            .filter(|s| &s.task_id == task_id)
            .collect();
        
        if task_sessions.is_empty() && !active.contains_key(task_id) {
            return None;
        }
        
        let total_time: u64 = task_sessions.iter().map(|s| s.total_duration).sum();
        let focused_time: u64 = task_sessions.iter().map(|s| s.active_duration).sum();
        let interruptions: u32 = task_sessions.iter().map(|s| s.interruptions).sum();
        
        // Add active session if exists
        let (total_time, focused_time, interruptions) = if let Some(active_session) = active.get(task_id) {
            (
                total_time + active_session.get_current_duration(),
                focused_time + active_session.get_focused_time(),
                interruptions + active_session.interruptions,
            )
        } else {
            (total_time, focused_time, interruptions)
        };
        
        let sessions_count = task_sessions.len() + if active.contains_key(task_id) { 1 } else { 0 };
        
        let avg_session_duration = if sessions_count > 0 {
            total_time / sessions_count as u64
        } else {
            0
        };
        
        let focus_ratio = if total_time > 0 {
            focused_time as f64 / total_time as f64
        } else {
            0.0
        };
        
        Some(TaskInsights {
            task_id: task_id.clone(),
            total_time,
            focused_time,
            sessions_count: sessions_count as u64,
            interruptions: interruptions as u64,
            avg_session_duration,
            focus_ratio,
            is_currently_active: active.contains_key(task_id),
        })
    }

    /// Get productivity report for date range
    pub async fn get_productivity_report(&self, start_date: &str, end_date: &str) -> ProductivityReport {
        let summaries = self.daily_summaries.read().await;
        
        let mut total_time = 0;
        let mut total_sessions = 0;
        let mut total_interruptions = 0;
        let mut daily_reports = Vec::new();

        for (date, summary) in summaries.iter() {
            if date.as_str() >= start_date && date.as_str() <= end_date {
                total_time += summary.total_active_time;
                total_sessions += summary.total_sessions;
                total_interruptions += summary.total_interruptions;
                daily_reports.push(summary.clone());
            }
        }

        ProductivityReport {
            period_start: parse_date_to_timestamp(start_date),
            period_end: parse_date_to_timestamp(end_date),
            total_time,
            total_sessions: total_sessions.into(),
            total_interruptions: total_interruptions.into(),
            total_focused_time: total_time, // For now, assume all time is focused in this legacy method
            total_break_time: 0,
            avg_session_duration: if total_sessions > 0 { total_time / total_sessions as u64 } else { 0 },
            focus_efficiency: 100.0, // Default to 100% for legacy method
            avg_interruptions_per_session: if total_sessions > 0 { total_interruptions as f64 / total_sessions as f64 } else { 0.0 },
            daily_breakdown: vec![], // Empty for legacy method
        }
    }

    /// Calculate productivity score for a session
    fn calculate_productivity_score(&self, session: &TrackingSession) -> f64 {
        if session.total_active_time == 0 {
            return 0.0;
        }

        let total_time = session.total_active_time + session.break_time;
        let active_ratio = session.total_active_time as f64 / total_time as f64;
        
        // Base score from active time ratio
        let mut score = active_ratio * 100.0;
        
        // Penalty for interruptions
        let interruption_penalty = (session.interruptions as f64) * 5.0;
        score = (score - interruption_penalty).max(0.0);
        
        // Bonus for longer focused sessions
        if session.total_active_time > 1800 { // 30+ minutes
            score += 10.0;
        }
        
        score.min(100.0)
    }

    /// Calculate productivity score for a period
    fn calculate_period_productivity_score(&self, summaries: &[DailySummary]) -> f64 {
        if summaries.is_empty() {
            return 0.0;
        }

        let total_score: f64 = summaries.iter().map(|s| s.productivity_score).sum();
        total_score / summaries.len() as f64
    }

    /// Update daily summary with completed session
    async fn update_daily_summary(&self, session: &CompletedSession) {
        let date = format_date_from_timestamp(session.started_at);
        
        let mut summaries = self.daily_summaries.write().await;
        let summary = summaries.entry(date.clone()).or_insert_with(|| DailySummary {
            date: date.clone(),
            total_active_time: 0,
            total_break_time: 0,
            total_sessions: 0,
            total_interruptions: 0,
            longest_session: 0,
            productivity_score: 0.0,
            task_breakdown: HashMap::new(),
        });

        summary.total_active_time += session.active_duration;
        summary.total_break_time += session.break_duration;
        summary.total_sessions += 1;
        summary.total_interruptions += session.interruptions;
        summary.longest_session = summary.longest_session.max(session.active_duration);

        // Update task breakdown
        *summary.task_breakdown.entry(session.task_id.clone()).or_insert(0) += session.active_duration;

        // Recalculate productivity score
        summary.productivity_score = session.productivity_score;
    }

    /// Update productivity metrics for a task
    async fn update_productivity_metrics(&self, task_id: &TaskId, session: &CompletedSession) {
        let mut metrics = self.productivity_metrics.write().await;
        let metric = metrics.entry(task_id.clone()).or_insert_with(|| ProductivityMetrics {
            task_id: task_id.clone(),
            total_tracked_time: 0,
            total_sessions: 0,
            average_session_length: 0,
            total_interruptions: 0,
            best_productivity_score: 0.0,
            average_productivity_score: 0.0,
            last_session_date: 0,
        });

        metric.total_tracked_time += session.active_duration;
        metric.total_sessions += 1;
        metric.average_session_length = metric.total_tracked_time / metric.total_sessions as u64;
        metric.total_interruptions += session.interruptions;
        metric.best_productivity_score = metric.best_productivity_score.max(session.productivity_score);
        metric.last_session_date = session.ended_at;

        // Update average productivity score
        metric.average_productivity_score = 
            (metric.average_productivity_score * (metric.total_sessions - 1) as f64 + session.productivity_score) 
            / metric.total_sessions as f64;
    }
}

/// Active tracking session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingSession {
    pub task_id: TaskId,
    pub started_at: u64,
    pub last_activity: u64,
    pub total_active_time: u64,
    pub break_time: u64,
    pub interruptions: u32,
    pub is_paused: bool,
    pub segments: Vec<TimeSegment>,
}

impl TrackingSession {
    /// Get current duration for active session
    pub fn get_current_duration(&self) -> u64 {
        if !self.is_paused {
            let current_time = current_timestamp();
            self.total_active_time + (current_time.saturating_sub(self.last_activity))
        } else {
            self.total_active_time
        }
    }

    /// Get focused time (active time minus interruptions)
    pub fn get_focused_time(&self) -> u64 {
        // Estimate focused time by subtracting interruption overhead
        let interruption_overhead = self.interruptions as u64 * 30; // 30 seconds per interruption
        if self.total_active_time > interruption_overhead {
            self.total_active_time - interruption_overhead
        } else {
            0
        }
    }
}

/// Completed tracking session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSession {
    pub task_id: TaskId,
    pub started_at: u64,
    pub ended_at: u64,
    pub total_duration: u64,
    pub active_duration: u64,
    pub break_duration: u64,
    pub interruptions: u32,
    pub segments: Vec<TimeSegment>,
    pub productivity_score: f64,
}

impl CompletedSession {
    /// Get total time for this session
    pub fn get_total_time(&self) -> u64 {
        self.total_duration
    }

    /// Get focused time (active time minus interruptions)
    pub fn get_focused_time(&self) -> u64 {
        // Estimate focused time by subtracting interruption overhead
        let interruption_overhead = self.interruptions as u64 * 30; // 30 seconds per interruption
        if self.active_duration > interruption_overhead {
            self.active_duration - interruption_overhead
        } else {
            0
        }
    }
}

/// Time segment within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSegment {
    pub started_at: u64,
    pub ended_at: u64,
    pub segment_type: SegmentType,
}

/// Type of time segment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentType {
    Active,
    Break,
    Interruption,
}

/// Task productivity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityMetrics {
    pub task_id: TaskId,
    pub total_tracked_time: u64,
    pub total_sessions: u32,
    pub average_session_length: u64,
    pub total_interruptions: u32,
    pub best_productivity_score: f64,
    pub average_productivity_score: f64,
    pub last_session_date: u64,
}

/// Daily summary of tracking activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: String,
    pub total_active_time: u64,
    pub total_break_time: u64,
    pub total_sessions: u32,
    pub total_interruptions: u32,
    pub longest_session: u64,
    pub productivity_score: f64,
    pub task_breakdown: HashMap<TaskId, u64>,
}


/// Tracker statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerStats {
    pub sessions_started: u64,
    pub sessions_completed: u64,
    pub total_tracked_time: u64,
    pub active_sessions: u32,
    pub created_at: u64,
}

/// Enhanced productivity report with detailed analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityReport {
    pub period_start: u64,
    pub period_end: u64,
    pub total_sessions: u64,
    pub total_time: u64,
    pub total_focused_time: u64,
    pub total_break_time: u64,
    pub total_interruptions: u64,
    pub avg_session_duration: u64,
    pub focus_efficiency: f64,
    pub avg_interruptions_per_session: f64,
    pub daily_breakdown: Vec<DailyStats>,
}

/// Daily productivity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: u64,
    pub total_time: u64,
    pub focused_time: u64,
    pub sessions: u64,
    pub interruptions: u64,
}

/// Task-specific insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInsights {
    pub task_id: TaskId,
    pub total_time: u64,
    pub focused_time: u64,
    pub sessions_count: u64,
    pub interruptions: u64,
    pub avg_session_duration: u64,
    pub focus_ratio: f64,
    pub is_currently_active: bool,
}

impl TrackerStats {
    fn new() -> Self {
        Self {
            sessions_started: 0,
            sessions_completed: 0,
            total_tracked_time: 0,
            active_sessions: 0,
            created_at: current_timestamp(),
        }
    }
}

/// Time tracker for simple time tracking operations
#[derive(Debug)]
pub struct TimeTracker {
    start_time: Option<u64>,
}

impl TimeTracker {
    pub fn new() -> Self {
        Self { start_time: None }
    }

    pub fn start(&mut self) {
        self.start_time = Some(current_timestamp());
    }

    pub fn stop(&mut self) -> Option<Duration> {
        if let Some(start) = self.start_time.take() {
            let duration = current_timestamp() - start;
            Some(Duration::from_secs(duration))
        } else {
            None
        }
    }

    pub fn elapsed(&self) -> Option<Duration> {
        if let Some(start) = self.start_time {
            let duration = current_timestamp() - start;
            Some(Duration::from_secs(duration))
        } else {
            None
        }
    }

    pub fn is_running(&self) -> bool {
        self.start_time.is_some()
    }
}

/// Format timestamp to date string (YYYY-MM-DD)
fn format_date_from_timestamp(timestamp: u64) -> String {
    let datetime = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
    let datetime = chrono::DateTime::<chrono::Utc>::from(datetime);
    datetime.format("%Y-%m-%d").to_string()
}

/// Parse date string to timestamp (simplified for now)
fn parse_date_to_timestamp(_date: &str) -> u64 {
    // For now, just return current timestamp
    // In a real implementation, this would parse the date string
    current_timestamp()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_task_tracker_creation() {
        let tracker = TaskTracker::new();
        let stats = tracker.get_stats().await;
        
        assert_eq!(stats.sessions_started, 0);
        assert_eq!(stats.sessions_completed, 0);
        assert_eq!(stats.total_tracked_time, 0);
    }

    #[tokio::test]
    async fn test_start_stop_tracking() {
        let tracker = TaskTracker::new();
        let task_id = "test-task-123".to_string();
        
        // Start tracking
        tracker.start_task(&task_id).await;
        
        let active_sessions = tracker.get_active_sessions().await;
        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].task_id, task_id);
        
        // Small delay to ensure time difference
        sleep(TokioDuration::from_millis(50)).await;
        
        // Stop tracking
        let duration = tracker.stop_task(&task_id).await;
        assert!(duration.is_some());
        assert!(duration.unwrap().as_millis() >= 10);
        
        let active_sessions = tracker.get_active_sessions().await;
        assert_eq!(active_sessions.len(), 0);
        
        let stats = tracker.get_stats().await;
        assert_eq!(stats.sessions_started, 1);
        assert_eq!(stats.sessions_completed, 1);
    }

    #[tokio::test]
    async fn test_pause_resume_tracking() {
        let tracker = TaskTracker::new();
        let task_id = "test-task-123".to_string();
        
        tracker.start_task(&task_id).await;
        
        let session = tracker.get_active_session(&task_id).await.unwrap();
        assert!(!session.is_paused);
        
        // Pause
        let paused = tracker.pause_task(&task_id).await;
        assert!(paused);
        
        let session = tracker.get_active_session(&task_id).await.unwrap();
        assert!(session.is_paused);
        
        // Resume
        let resumed = tracker.resume_task(&task_id).await;
        assert!(resumed);
        
        let session = tracker.get_active_session(&task_id).await.unwrap();
        assert!(!session.is_paused);
    }

    #[tokio::test]
    async fn test_record_interruption() {
        let tracker = TaskTracker::new();
        let task_id = "test-task-123".to_string();
        
        tracker.start_task(&task_id).await;
        tracker.record_interruption(&task_id, 60).await; // 1 minute interruption
        
        let session = tracker.get_active_session(&task_id).await.unwrap();
        assert_eq!(session.interruptions, 1);
        assert_eq!(session.break_time, 60);
        assert_eq!(session.segments.len(), 1);
        assert_eq!(session.segments[0].segment_type, SegmentType::Interruption);
    }

    #[tokio::test]
    async fn test_productivity_metrics() {
        let tracker = TaskTracker::new();
        let task_id = "test-task-123".to_string();
        
        // Start and stop a session
        tracker.start_task(&task_id).await;
        sleep(TokioDuration::from_millis(50)).await;
        tracker.stop_task(&task_id).await;
        
        let metrics = tracker.get_productivity_metrics(&task_id).await;
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.task_id, task_id);
        assert_eq!(metrics.total_sessions, 1);
        assert!(metrics.total_tracked_time > 0);
        assert!(metrics.average_productivity_score > 0.0);
    }

    #[tokio::test]
    async fn test_time_tracker() {
        let mut timer = TimeTracker::new();
        
        assert!(!timer.is_running());
        assert!(timer.elapsed().is_none());
        
        timer.start();
        assert!(timer.is_running());
        assert!(timer.elapsed().is_some());
        
        sleep(TokioDuration::from_millis(50)).await;
        
        let duration = timer.stop();
        assert!(duration.is_some());
        assert!(duration.unwrap().as_millis() >= 10);
        assert!(!timer.is_running());
    }

    #[test]
    fn test_format_date_from_timestamp() {
        let timestamp = 1640995200; // 2022-01-01 00:00:00 UTC
        let date = format_date_from_timestamp(timestamp);
        assert_eq!(date, "2022-01-01");
    }
}