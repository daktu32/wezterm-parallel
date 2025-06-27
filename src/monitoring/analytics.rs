// WezTerm Multi-Process Development Framework - Analytics System
// Provides advanced analytics and insights for system performance and usage

use super::{SystemMetrics, Alert, HealthCheck};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Analytics manager for comprehensive system analysis
pub struct AnalyticsManager {
    /// Historical metrics data
    metrics_history: Arc<RwLock<Vec<SystemMetrics>>>,
    
    /// Alert history for pattern analysis
    alert_history: Arc<RwLock<Vec<Alert>>>,
    
    /// Health check history
    health_history: Arc<RwLock<Vec<HealthCheck>>>,
    
    /// Performance baselines
    baselines: Arc<RwLock<PerformanceBaselines>>,
    
    /// Usage patterns
    usage_patterns: Arc<RwLock<UsagePatterns>>,
}

/// Performance baselines for comparison
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceBaselines {
    /// Average CPU usage baseline
    pub cpu_baseline: f64,
    
    /// Average memory usage baseline
    pub memory_baseline: u64,
    
    /// Average disk usage baseline
    pub disk_baseline: u64,
    
    /// Response time baselines by component
    pub response_time_baselines: HashMap<String, u64>,
    
    /// Baseline calculation timestamp
    pub calculated_at: u64,
    
    /// Number of samples used for baseline
    pub sample_count: usize,
}

/// Usage patterns analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsagePatterns {
    /// Peak usage hours
    pub peak_hours: Vec<u8>,
    
    /// Average session duration
    pub avg_session_duration: u64,
    
    /// Most used components
    pub component_usage: HashMap<String, u64>,
    
    /// Workspace usage patterns
    pub workspace_patterns: HashMap<String, WorkspaceUsage>,
    
    /// Error frequency patterns
    pub error_patterns: HashMap<String, u32>,
}

/// Workspace usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceUsage {
    pub total_time: u64,
    pub session_count: u32,
    pub avg_session_duration: u64,
    pub last_used: u64,
    pub task_count: u32,
}

/// Comprehensive analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    /// Report generation timestamp
    pub generated_at: u64,
    
    /// Report time range
    pub time_range: TimeRange,
    
    /// Performance analysis
    pub performance: PerformanceAnalysis,
    
    /// Reliability analysis
    pub reliability: ReliabilityAnalysis,
    
    /// Usage analysis
    pub usage: UsageAnalysis,
    
    /// Trend analysis
    pub trends: TrendAnalysis,
    
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
    
    /// Executive summary
    pub summary: ExecutiveSummary,
}

/// Time range for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: u64,
    pub end: u64,
    pub duration_hours: u64,
}

/// Performance analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// Average performance metrics
    pub averages: PerformanceAverages,
    
    /// Peak performance metrics
    pub peaks: PerformancePeaks,
    
    /// Performance variance
    pub variance: PerformanceVariance,
    
    /// Performance score (0-100)
    pub performance_score: f64,
    
    /// Bottleneck identification
    pub bottlenecks: Vec<Bottleneck>,
}

/// Average performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAverages {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub disk_usage: u64,
    pub response_time: u64,
    pub throughput: f64,
}

/// Peak performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePeaks {
    pub max_cpu_usage: f64,
    pub max_memory_usage: u64,
    pub max_disk_usage: u64,
    pub max_response_time: u64,
    pub peak_timestamp: u64,
}

/// Performance variance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceVariance {
    pub cpu_variance: f64,
    pub memory_variance: f64,
    pub stability_score: f64,
}

/// System bottleneck identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub metric: String,
    pub severity: BottleneckSeverity,
    pub impact_score: f64,
    pub recommendation: String,
}

/// Bottleneck severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Reliability analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityAnalysis {
    /// System uptime percentage
    pub uptime_percentage: f64,
    
    /// Mean time between failures (MTBF)
    pub mtbf_hours: f64,
    
    /// Mean time to recovery (MTTR)
    pub mttr_minutes: f64,
    
    /// Error rate analysis
    pub error_rates: HashMap<String, f64>,
    
    /// Service level indicators
    pub sli_metrics: SLIMetrics,
    
    /// Reliability score (0-100)
    pub reliability_score: f64,
}

/// Service Level Indicator metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLIMetrics {
    pub availability: f64,
    pub response_time_p95: u64,
    pub response_time_p99: u64,
    pub error_rate: f64,
    pub throughput: f64,
}

/// Usage analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalysis {
    /// Active users/sessions
    pub active_sessions: u32,
    
    /// Feature usage statistics
    pub feature_usage: HashMap<String, FeatureUsage>,
    
    /// Resource utilization
    pub resource_utilization: ResourceUtilization,
    
    /// User behavior patterns
    pub behavior_patterns: BehaviorPatterns,
}

/// Feature usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUsage {
    pub usage_count: u64,
    pub unique_sessions: u32,
    pub avg_duration: u64,
    pub success_rate: f64,
}

/// Resource utilization analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub disk_utilization: f64,
    pub network_utilization: f64,
    pub efficiency_score: f64,
}

/// User behavior patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPatterns {
    pub peak_usage_hours: Vec<u8>,
    pub avg_session_duration: u64,
    pub common_workflows: Vec<Workflow>,
    pub abandonment_points: Vec<String>,
}

/// Common workflow pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub steps: Vec<String>,
    pub frequency: u32,
    pub success_rate: f64,
    pub avg_duration: u64,
}

/// Trend analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Performance trends
    pub performance_trends: Vec<TrendData>,
    
    /// Usage trends
    pub usage_trends: Vec<TrendData>,
    
    /// Error trends
    pub error_trends: Vec<TrendData>,
    
    /// Capacity planning insights
    pub capacity_insights: CapacityInsights,
    
    /// Forecast predictions
    pub forecasts: Vec<Forecast>,
}

/// Trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub timestamp: u64,
    pub metric: String,
    pub value: f64,
    pub trend_direction: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Capacity planning insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityInsights {
    pub current_capacity_usage: f64,
    pub projected_capacity_exhaustion: Option<u64>,
    pub scaling_recommendations: Vec<String>,
    pub resource_optimization_opportunities: Vec<String>,
}

/// Forecast prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub metric: String,
    pub prediction_timestamp: u64,
    pub predicted_value: f64,
    pub confidence_interval: (f64, f64),
    pub confidence_level: f64,
}

/// System recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub effort: String,
    pub implementation_steps: Vec<String>,
    pub expected_improvement: Option<f64>,
}

/// Recommendation categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Performance,
    Reliability,
    Security,
    Capacity,
    Cost,
    UserExperience,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Executive summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub overall_health_score: f64,
    pub key_achievements: Vec<String>,
    pub critical_issues: Vec<String>,
    pub improvement_opportunities: Vec<String>,
    pub resource_efficiency: f64,
    pub user_satisfaction_score: f64,
}

impl AnalyticsManager {
    /// Create new analytics manager
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            health_history: Arc::new(RwLock::new(Vec::new())),
            baselines: Arc::new(RwLock::new(PerformanceBaselines::default())),
            usage_patterns: Arc::new(RwLock::new(UsagePatterns::default())),
        }
    }
    
    /// Add metrics data for analysis
    pub async fn add_metrics(&self, metrics: SystemMetrics) {
        let mut history = self.metrics_history.write().await;
        history.push(metrics);
        
        // Keep only last 1000 entries to manage memory
        if history.len() > 1000 {
            history.drain(0..100);
        }
        
        // Update baselines periodically
        if history.len() % 100 == 0 {
            self.update_baselines().await;
        }
    }
    
    /// Add alert data for analysis
    pub async fn add_alert(&self, alert: Alert) {
        let mut history = self.alert_history.write().await;
        history.push(alert);
        
        // Keep only last 500 alerts
        if history.len() > 500 {
            history.drain(0..50);
        }
    }
    
    /// Add health check data for analysis
    pub async fn add_health_check(&self, health_check: HealthCheck) {
        let mut history = self.health_history.write().await;
        history.push(health_check);
        
        // Keep only last 200 health checks
        if history.len() > 200 {
            history.drain(0..20);
        }
    }
    
    /// Generate comprehensive analytics report
    pub async fn generate_report(&self, time_range: TimeRange) -> AnalyticsReport {
        let generated_at = current_timestamp();
        
        // Gather data within time range
        let metrics = self.get_metrics_in_range(&time_range).await;
        let alerts = self.get_alerts_in_range(&time_range).await;
        let health_checks = self.get_health_checks_in_range(&time_range).await;
        
        // Perform analyses
        let performance = self.analyze_performance(&metrics).await;
        let reliability = self.analyze_reliability(&metrics, &alerts, &health_checks).await;
        let usage = self.analyze_usage(&metrics).await;
        let trends = self.analyze_trends(&metrics, &alerts).await;
        let recommendations = self.generate_recommendations(&performance, &reliability, &usage, &trends).await;
        let summary = self.generate_executive_summary(&performance, &reliability, &usage, &recommendations).await;
        
        AnalyticsReport {
            generated_at,
            time_range,
            performance,
            reliability,
            usage,
            trends,
            recommendations,
            summary,
        }
    }
    
    /// Get metrics within time range
    async fn get_metrics_in_range(&self, time_range: &TimeRange) -> Vec<SystemMetrics> {
        let history = self.metrics_history.read().await;
        history.iter()
            .filter(|m| m.timestamp >= time_range.start && m.timestamp <= time_range.end)
            .cloned()
            .collect()
    }
    
    /// Get alerts within time range
    async fn get_alerts_in_range(&self, time_range: &TimeRange) -> Vec<Alert> {
        let history = self.alert_history.read().await;
        history.iter()
            .filter(|a| a.timestamp >= time_range.start && a.timestamp <= time_range.end)
            .cloned()
            .collect()
    }
    
    /// Get health checks within time range
    async fn get_health_checks_in_range(&self, time_range: &TimeRange) -> Vec<HealthCheck> {
        let history = self.health_history.read().await;
        history.iter()
            .filter(|h| h.timestamp >= time_range.start && h.timestamp <= time_range.end)
            .cloned()
            .collect()
    }
    
    /// Analyze performance metrics
    async fn analyze_performance(&self, metrics: &[SystemMetrics]) -> PerformanceAnalysis {
        if metrics.is_empty() {
            return PerformanceAnalysis {
                averages: PerformanceAverages {
                    cpu_usage: 0.0,
                    memory_usage: 0,
                    disk_usage: 0,
                    response_time: 0,
                    throughput: 0.0,
                },
                peaks: PerformancePeaks {
                    max_cpu_usage: 0.0,
                    max_memory_usage: 0,
                    max_disk_usage: 0,
                    max_response_time: 0,
                    peak_timestamp: 0,
                },
                variance: PerformanceVariance {
                    cpu_variance: 0.0,
                    memory_variance: 0.0,
                    stability_score: 0.0,
                },
                performance_score: 0.0,
                bottlenecks: vec![],
            };
        }
        
        // Calculate averages
        let avg_cpu = metrics.iter().map(|m| m.cpu_usage).sum::<f64>() / metrics.len() as f64;
        let avg_memory = metrics.iter().map(|m| m.memory_usage).sum::<u64>() / metrics.len() as u64;
        let avg_disk = metrics.iter().map(|m| m.disk_usage).sum::<u64>() / metrics.len() as u64;
        
        // Find peaks
        let max_cpu = metrics.iter().map(|m| m.cpu_usage).fold(0.0, f64::max);
        let max_memory = metrics.iter().map(|m| m.memory_usage).max().unwrap_or(0);
        let max_disk = metrics.iter().map(|m| m.disk_usage).max().unwrap_or(0);
        let peak_timestamp = metrics.iter()
            .max_by_key(|m| m.cpu_usage as u64)
            .map(|m| m.timestamp)
            .unwrap_or(0);
        
        // Calculate variance
        let cpu_variance = if metrics.len() > 1 {
            metrics.iter()
                .map(|m| (m.cpu_usage - avg_cpu).powi(2))
                .sum::<f64>() / (metrics.len() - 1) as f64
        } else {
            0.0
        };
        
        let memory_variance = if metrics.len() > 1 {
            metrics.iter()
                .map(|m| (m.memory_usage as f64 - avg_memory as f64).powi(2))
                .sum::<f64>() / (metrics.len() - 1) as f64
        } else {
            0.0
        };
        
        // Calculate performance score (simplified)
        let performance_score = if max_cpu < 80.0 && avg_cpu < 50.0 {
            90.0 - (avg_cpu / 100.0) * 30.0
        } else {
            60.0 - (avg_cpu / 100.0) * 50.0
        };
        
        // Identify bottlenecks
        let mut bottlenecks = Vec::new();
        
        if avg_cpu > 70.0 {
            bottlenecks.push(Bottleneck {
                component: "CPU".to_string(),
                metric: "average_usage".to_string(),
                severity: if avg_cpu > 90.0 { BottleneckSeverity::Critical } else { BottleneckSeverity::High },
                impact_score: avg_cpu,
                recommendation: "Consider optimizing CPU-intensive operations or scaling horizontally".to_string(),
            });
        }
        
        let memory_usage_pct = if avg_memory > 0 { (avg_memory as f64 / (8 * 1024 * 1024 * 1024) as f64) * 100.0 } else { 0.0 };
        if memory_usage_pct > 80.0 {
            bottlenecks.push(Bottleneck {
                component: "Memory".to_string(),
                metric: "average_usage".to_string(),
                severity: if memory_usage_pct > 95.0 { BottleneckSeverity::Critical } else { BottleneckSeverity::High },
                impact_score: memory_usage_pct,
                recommendation: "Consider increasing memory or optimizing memory usage patterns".to_string(),
            });
        }
        
        PerformanceAnalysis {
            averages: PerformanceAverages {
                cpu_usage: avg_cpu,
                memory_usage: avg_memory,
                disk_usage: avg_disk,
                response_time: 100, // Placeholder
                throughput: 10.0,   // Placeholder
            },
            peaks: PerformancePeaks {
                max_cpu_usage: max_cpu,
                max_memory_usage: max_memory,
                max_disk_usage: max_disk,
                max_response_time: 500, // Placeholder
                peak_timestamp,
            },
            variance: PerformanceVariance {
                cpu_variance,
                memory_variance,
                stability_score: 100.0 - (cpu_variance + memory_variance / 1000000.0).min(100.0),
            },
            performance_score,
            bottlenecks,
        }
    }
    
    /// Analyze system reliability
    async fn analyze_reliability(&self, _metrics: &[SystemMetrics], alerts: &[Alert], health_checks: &[HealthCheck]) -> ReliabilityAnalysis {
        // Calculate uptime based on health checks
        let healthy_checks = health_checks.iter()
            .filter(|h| h.overall_status == super::HealthStatus::Healthy)
            .count();
        
        let uptime_percentage = if !health_checks.is_empty() {
            (healthy_checks as f64 / health_checks.len() as f64) * 100.0
        } else {
            100.0
        };
        
        // Calculate error rates
        let mut error_rates = HashMap::new();
        for alert in alerts {
            if alert.severity == super::AlertSeverity::Error || alert.severity == super::AlertSeverity::Critical {
                *error_rates.entry(alert.category.clone()).or_insert(0.0) += 1.0;
            }
        }
        
        let reliability_score = uptime_percentage * 0.7 + (100.0 - error_rates.values().sum::<f64>().min(100.0)) * 0.3;
        
        ReliabilityAnalysis {
            uptime_percentage,
            mtbf_hours: 168.0, // Placeholder: 1 week
            mttr_minutes: 5.0,  // Placeholder: 5 minutes
            error_rates,
            sli_metrics: SLIMetrics {
                availability: uptime_percentage / 100.0,
                response_time_p95: 200,
                response_time_p99: 500,
                error_rate: 0.1,
                throughput: 100.0,
            },
            reliability_score,
        }
    }
    
    /// Analyze usage patterns
    async fn analyze_usage(&self, metrics: &[SystemMetrics]) -> UsageAnalysis {
        let active_sessions = metrics.len() as u32; // Simplified
        
        let mut feature_usage = HashMap::new();
        feature_usage.insert("workspace_management".to_string(), FeatureUsage {
            usage_count: 100,
            unique_sessions: 10,
            avg_duration: 1800,
            success_rate: 95.0,
        });
        
        let resource_utilization = ResourceUtilization {
            cpu_utilization: if !metrics.is_empty() {
                metrics.iter().map(|m| m.cpu_usage).sum::<f64>() / metrics.len() as f64
            } else { 0.0 },
            memory_utilization: 75.0, // Placeholder
            disk_utilization: 60.0,   // Placeholder
            network_utilization: 30.0, // Placeholder
            efficiency_score: 80.0,
        };
        
        let behavior_patterns = BehaviorPatterns {
            peak_usage_hours: vec![9, 10, 11, 14, 15, 16], // 9-11 AM, 2-4 PM
            avg_session_duration: 3600, // 1 hour
            common_workflows: vec![
                Workflow {
                    name: "Development Session".to_string(),
                    steps: vec!["create_workspace".to_string(), "start_tasks".to_string(), "monitor_progress".to_string()],
                    frequency: 50,
                    success_rate: 92.0,
                    avg_duration: 7200, // 2 hours
                }
            ],
            abandonment_points: vec!["task_creation".to_string()],
        };
        
        UsageAnalysis {
            active_sessions,
            feature_usage,
            resource_utilization,
            behavior_patterns,
        }
    }
    
    /// Analyze trends
    async fn analyze_trends(&self, metrics: &[SystemMetrics], alerts: &[Alert]) -> TrendAnalysis {
        let mut performance_trends = Vec::new();
        let mut error_trends = Vec::new();
        
        // Simple trend analysis - in production, this would use more sophisticated algorithms
        for (i, metric) in metrics.iter().enumerate() {
            if i % 10 == 0 { // Sample every 10th metric
                performance_trends.push(TrendData {
                    timestamp: metric.timestamp,
                    metric: "cpu_usage".to_string(),
                    value: metric.cpu_usage,
                    trend_direction: TrendDirection::Stable, // Simplified
                });
            }
        }
        
        for alert in alerts {
            error_trends.push(TrendData {
                timestamp: alert.timestamp,
                metric: "alert_count".to_string(),
                value: 1.0,
                trend_direction: TrendDirection::Stable, // Simplified
            });
        }
        
        let capacity_insights = CapacityInsights {
            current_capacity_usage: 65.0,
            projected_capacity_exhaustion: None,
            scaling_recommendations: vec![
                "Monitor CPU usage trends".to_string(),
                "Consider memory optimization".to_string(),
            ],
            resource_optimization_opportunities: vec![
                "Implement process pooling".to_string(),
                "Optimize disk I/O operations".to_string(),
            ],
        };
        
        TrendAnalysis {
            performance_trends,
            usage_trends: vec![], // Placeholder
            error_trends,
            capacity_insights,
            forecasts: vec![], // Placeholder
        }
    }
    
    /// Generate recommendations
    async fn generate_recommendations(
        &self,
        performance: &PerformanceAnalysis,
        reliability: &ReliabilityAnalysis,
        _usage: &UsageAnalysis,
        _trends: &TrendAnalysis,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();
        
        // Performance recommendations
        if performance.performance_score < 70.0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::High,
                title: "Optimize system performance".to_string(),
                description: "System performance is below optimal levels".to_string(),
                impact: "Improved user experience and system efficiency".to_string(),
                effort: "Medium".to_string(),
                implementation_steps: vec![
                    "Profile CPU-intensive operations".to_string(),
                    "Implement caching strategies".to_string(),
                    "Optimize database queries".to_string(),
                ],
                expected_improvement: Some(15.0),
            });
        }
        
        // Reliability recommendations
        if reliability.reliability_score < 95.0 {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Reliability,
                priority: RecommendationPriority::Critical,
                title: "Improve system reliability".to_string(),
                description: "System reliability is below target SLA".to_string(),
                impact: "Reduced downtime and improved user trust".to_string(),
                effort: "High".to_string(),
                implementation_steps: vec![
                    "Implement circuit breaker patterns".to_string(),
                    "Add comprehensive monitoring".to_string(),
                    "Improve error handling".to_string(),
                ],
                expected_improvement: Some(10.0),
            });
        }
        
        recommendations
    }
    
    /// Generate executive summary
    async fn generate_executive_summary(
        &self,
        performance: &PerformanceAnalysis,
        reliability: &ReliabilityAnalysis,
        usage: &UsageAnalysis,
        recommendations: &[Recommendation],
    ) -> ExecutiveSummary {
        let overall_health_score = (performance.performance_score + reliability.reliability_score + usage.resource_utilization.efficiency_score) / 3.0;
        
        let key_achievements = vec![
            "Maintained stable system operation".to_string(),
            "Successfully processed user requests".to_string(),
        ];
        
        let critical_issues: Vec<String> = recommendations.iter()
            .filter(|r| matches!(r.priority, RecommendationPriority::Critical))
            .map(|r| r.title.clone())
            .collect();
        
        let improvement_opportunities: Vec<String> = recommendations.iter()
            .filter(|r| matches!(r.priority, RecommendationPriority::High | RecommendationPriority::Medium))
            .map(|r| r.title.clone())
            .take(3)
            .collect();
        
        ExecutiveSummary {
            overall_health_score,
            key_achievements,
            critical_issues,
            improvement_opportunities,
            resource_efficiency: usage.resource_utilization.efficiency_score,
            user_satisfaction_score: 85.0, // Placeholder
        }
    }
    
    /// Update performance baselines
    async fn update_baselines(&self) {
        let metrics = self.metrics_history.read().await;
        
        if metrics.len() < 10 {
            return; // Need at least 10 samples
        }
        
        let recent_metrics: Vec<_> = metrics.iter().rev().take(50).collect();
        
        let cpu_baseline = recent_metrics.iter().map(|m| m.cpu_usage).sum::<f64>() / recent_metrics.len() as f64;
        let memory_baseline = recent_metrics.iter().map(|m| m.memory_usage).sum::<u64>() / recent_metrics.len() as u64;
        let disk_baseline = recent_metrics.iter().map(|m| m.disk_usage).sum::<u64>() / recent_metrics.len() as u64;
        
        let mut baselines = self.baselines.write().await;
        baselines.cpu_baseline = cpu_baseline;
        baselines.memory_baseline = memory_baseline;
        baselines.disk_baseline = disk_baseline;
        baselines.calculated_at = current_timestamp();
        baselines.sample_count = recent_metrics.len();
    }
    
    /// Get current baselines
    pub async fn get_baselines(&self) -> PerformanceBaselines {
        self.baselines.read().await.clone()
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Default for AnalyticsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_analytics_manager_creation() {
        let manager = AnalyticsManager::new();
        let baselines = manager.get_baselines().await;
        assert_eq!(baselines.sample_count, 0);
    }
    
    #[tokio::test]
    async fn test_metrics_addition() {
        let manager = AnalyticsManager::new();
        
        let metrics = SystemMetrics {
            timestamp: current_timestamp(),
            cpu_usage: 50.0,
            memory_usage: 1024 * 1024 * 1024, // 1GB
            memory_available: 3 * 1024 * 1024 * 1024, // 3GB
            disk_usage: 10 * 1024 * 1024 * 1024, // 10GB
            disk_available: 90 * 1024 * 1024 * 1024, // 90GB
            network_io: crate::monitoring::NetworkIO {
                bytes_received: 1000,
                bytes_sent: 2000,
                packets_received: 10,
                packets_sent: 20,
            },
            process_metrics: std::collections::HashMap::new(),
        };
        
        manager.add_metrics(metrics).await;
        
        let history = manager.metrics_history.read().await;
        assert_eq!(history.len(), 1);
    }
}