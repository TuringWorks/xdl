//! Execution statistics and profiling for XDL-AMP
//!
//! Tracks performance metrics, operation counts, and provides
//! decision data for GPU vs CPU dispatch.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Operation types tracked by the statistics system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpType {
    Add,
    Sub,
    Mul,
    Div,
    Sin,
    Cos,
    Exp,
    Log,
    Sqrt,
    Pow,
    MatMul,
    Sum,
    Max,
    Min,
    MemCopyH2D, // Host to Device
    MemCopyD2H, // Device to Host
    BufferAlloc,
    BufferFree,
}

impl OpType {
    pub fn name(&self) -> &'static str {
        match self {
            OpType::Add => "Add",
            OpType::Sub => "Sub",
            OpType::Mul => "Mul",
            OpType::Div => "Div",
            OpType::Sin => "Sin",
            OpType::Cos => "Cos",
            OpType::Exp => "Exp",
            OpType::Log => "Log",
            OpType::Sqrt => "Sqrt",
            OpType::Pow => "Pow",
            OpType::MatMul => "MatMul",
            OpType::Sum => "Sum",
            OpType::Max => "Max",
            OpType::Min => "Min",
            OpType::MemCopyH2D => "H2D Copy",
            OpType::MemCopyD2H => "D2H Copy",
            OpType::BufferAlloc => "Buffer Alloc",
            OpType::BufferFree => "Buffer Free",
        }
    }
}

/// Execution layer where operation was performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionLayer {
    Cpu,
    GpuCompute,
    GpuMemory,
    Cache,
}

impl ExecutionLayer {
    pub fn name(&self) -> &'static str {
        match self {
            ExecutionLayer::Cpu => "CPU",
            ExecutionLayer::GpuCompute => "GPU Compute",
            ExecutionLayer::GpuMemory => "GPU Memory",
            ExecutionLayer::Cache => "Cache",
        }
    }
}

/// Statistics for a single operation type
#[derive(Debug, Default)]
pub struct OpStats {
    pub count: AtomicU64,
    pub total_time_ns: AtomicU64,
    pub total_elements: AtomicU64,
    pub total_bytes: AtomicU64,
    pub gpu_executions: AtomicU64,
    pub cpu_executions: AtomicU64,
    pub cache_hits: AtomicU64,
}

impl OpStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, time_ns: u64, elements: u64, bytes: u64, layer: ExecutionLayer) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_time_ns.fetch_add(time_ns, Ordering::Relaxed);
        self.total_elements.fetch_add(elements, Ordering::Relaxed);
        self.total_bytes.fetch_add(bytes, Ordering::Relaxed);

        match layer {
            ExecutionLayer::Cpu => {
                self.cpu_executions.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionLayer::GpuCompute | ExecutionLayer::GpuMemory => {
                self.gpu_executions.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionLayer::Cache => {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn avg_time_us(&self) -> f64 {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        let total_ns = self.total_time_ns.load(Ordering::Relaxed);
        (total_ns as f64 / count as f64) / 1000.0
    }

    pub fn throughput_meps(&self) -> f64 {
        // Million elements per second
        let total_ns = self.total_time_ns.load(Ordering::Relaxed);
        if total_ns == 0 {
            return 0.0;
        }
        let total_elements = self.total_elements.load(Ordering::Relaxed);
        (total_elements as f64 / (total_ns as f64 / 1_000_000_000.0)) / 1_000_000.0
    }

    pub fn bandwidth_gbps(&self) -> f64 {
        // Gigabytes per second
        let total_ns = self.total_time_ns.load(Ordering::Relaxed);
        if total_ns == 0 {
            return 0.0;
        }
        let total_bytes = self.total_bytes.load(Ordering::Relaxed);
        (total_bytes as f64 / (total_ns as f64 / 1_000_000_000.0)) / 1_000_000_000.0
    }
}

/// Dispatch decision record
#[derive(Debug, Clone)]
pub struct DispatchDecision {
    pub op: OpType,
    pub elements: usize,
    pub chosen_layer: ExecutionLayer,
    pub reason: &'static str,
    pub timestamp: Instant,
}

/// Global execution statistics
pub struct ExecutionStats {
    /// Per-operation statistics
    op_stats: HashMap<OpType, Arc<OpStats>>,
    /// Recent dispatch decisions (ring buffer)
    dispatch_history: RwLock<Vec<DispatchDecision>>,
    /// Maximum history entries
    max_history: usize,
    /// Session start time
    session_start: Instant,
    /// Backend name
    backend_name: RwLock<String>,
    /// GPU memory allocated
    gpu_memory_allocated: AtomicU64,
    /// GPU memory peak
    gpu_memory_peak: AtomicU64,
    /// Cache memory used
    cache_memory_used: AtomicU64,
    /// Total operations
    total_ops: AtomicU64,
    /// Enabled flag
    enabled: std::sync::atomic::AtomicBool,
}

impl ExecutionStats {
    pub fn new() -> Self {
        let mut op_stats = HashMap::new();

        // Initialize stats for all operation types
        for op in [
            OpType::Add,
            OpType::Sub,
            OpType::Mul,
            OpType::Div,
            OpType::Sin,
            OpType::Cos,
            OpType::Exp,
            OpType::Log,
            OpType::Sqrt,
            OpType::Pow,
            OpType::MatMul,
            OpType::Sum,
            OpType::Max,
            OpType::Min,
            OpType::MemCopyH2D,
            OpType::MemCopyD2H,
            OpType::BufferAlloc,
            OpType::BufferFree,
        ] {
            op_stats.insert(op, Arc::new(OpStats::new()));
        }

        Self {
            op_stats,
            dispatch_history: RwLock::new(Vec::with_capacity(1000)),
            max_history: 1000,
            session_start: Instant::now(),
            backend_name: RwLock::new("Unknown".to_string()),
            gpu_memory_allocated: AtomicU64::new(0),
            gpu_memory_peak: AtomicU64::new(0),
            cache_memory_used: AtomicU64::new(0),
            total_ops: AtomicU64::new(0),
            enabled: std::sync::atomic::AtomicBool::new(true),
        }
    }

    pub fn set_backend(&self, name: &str) {
        *self.backend_name.write().unwrap() = name.to_string();
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Record an operation execution
    pub fn record_op(
        &self,
        op: OpType,
        time: Duration,
        elements: usize,
        bytes: usize,
        layer: ExecutionLayer,
    ) {
        if !self.is_enabled() {
            return;
        }

        if let Some(stats) = self.op_stats.get(&op) {
            stats.record(time.as_nanos() as u64, elements as u64, bytes as u64, layer);
        }
        self.total_ops.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a dispatch decision
    pub fn record_dispatch(
        &self,
        op: OpType,
        elements: usize,
        layer: ExecutionLayer,
        reason: &'static str,
    ) {
        if !self.is_enabled() {
            return;
        }

        let decision = DispatchDecision {
            op,
            elements,
            chosen_layer: layer,
            reason,
            timestamp: Instant::now(),
        };

        let mut history = self.dispatch_history.write().unwrap();
        if history.len() >= self.max_history {
            history.remove(0);
        }
        history.push(decision);
    }

    /// Record GPU memory allocation
    pub fn record_gpu_alloc(&self, bytes: usize) {
        let current = self
            .gpu_memory_allocated
            .fetch_add(bytes as u64, Ordering::Relaxed)
            + bytes as u64;
        let mut peak = self.gpu_memory_peak.load(Ordering::Relaxed);
        while current > peak {
            match self.gpu_memory_peak.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(p) => peak = p,
            }
        }
    }

    /// Record GPU memory free
    pub fn record_gpu_free(&self, bytes: usize) {
        self.gpu_memory_allocated
            .fetch_sub(bytes as u64, Ordering::Relaxed);
    }

    /// Record cache memory change
    pub fn record_cache_memory(&self, bytes: i64) {
        if bytes >= 0 {
            self.cache_memory_used
                .fetch_add(bytes as u64, Ordering::Relaxed);
        } else {
            self.cache_memory_used
                .fetch_sub((-bytes) as u64, Ordering::Relaxed);
        }
    }

    /// Get statistics for an operation type
    pub fn get_op_stats(&self, op: OpType) -> Option<&Arc<OpStats>> {
        self.op_stats.get(&op)
    }

    /// Get session duration
    pub fn session_duration(&self) -> Duration {
        self.session_start.elapsed()
    }

    /// Generate a statistics report
    pub fn report(&self) -> StatsReport {
        let backend = self.backend_name.read().unwrap().clone();
        let duration = self.session_duration();

        let mut op_summaries = Vec::new();
        for (op, stats) in &self.op_stats {
            let count = stats.count.load(Ordering::Relaxed);
            if count > 0 {
                op_summaries.push(OpSummary {
                    op: *op,
                    count,
                    avg_time_us: stats.avg_time_us(),
                    throughput_meps: stats.throughput_meps(),
                    bandwidth_gbps: stats.bandwidth_gbps(),
                    gpu_pct: if count > 0 {
                        (stats.gpu_executions.load(Ordering::Relaxed) as f64 / count as f64) * 100.0
                    } else {
                        0.0
                    },
                    cache_hit_pct: if count > 0 {
                        (stats.cache_hits.load(Ordering::Relaxed) as f64 / count as f64) * 100.0
                    } else {
                        0.0
                    },
                });
            }
        }

        // Sort by count descending
        op_summaries.sort_by(|a, b| b.count.cmp(&a.count));

        StatsReport {
            backend,
            duration,
            total_ops: self.total_ops.load(Ordering::Relaxed),
            gpu_memory_current: self.gpu_memory_allocated.load(Ordering::Relaxed),
            gpu_memory_peak: self.gpu_memory_peak.load(Ordering::Relaxed),
            cache_memory: self.cache_memory_used.load(Ordering::Relaxed),
            op_summaries,
        }
    }

    /// Format report as string for display
    pub fn format_report(&self) -> String {
        let report = self.report();
        let mut output = String::new();

        output.push_str(
            "\n╔══════════════════════════════════════════════════════════════════════════════╗\n",
        );
        output.push_str(
            "║                        XDL-AMP EXECUTION STATISTICS                          ║\n",
        );
        output.push_str(
            "╠══════════════════════════════════════════════════════════════════════════════╣\n",
        );

        output.push_str(&format!("║ Backend: {:<67} ║\n", report.backend));
        output.push_str(&format!(
            "║ Session Duration: {:<58} ║\n",
            format!("{:.2}s", report.duration.as_secs_f64())
        ));
        output.push_str(&format!(
            "║ Total Operations: {:<58} ║\n",
            format_number(report.total_ops)
        ));

        output.push_str(
            "╠══════════════════════════════════════════════════════════════════════════════╣\n",
        );
        output.push_str(
            "║ MEMORY                                                                       ║\n",
        );
        output.push_str(
            "╠══════════════════════════════════════════════════════════════════════════════╣\n",
        );
        output.push_str(&format!(
            "║ GPU Memory (Current): {:<54} ║\n",
            format_bytes(report.gpu_memory_current)
        ));
        output.push_str(&format!(
            "║ GPU Memory (Peak):    {:<54} ║\n",
            format_bytes(report.gpu_memory_peak)
        ));
        output.push_str(&format!(
            "║ Cache Memory:         {:<54} ║\n",
            format_bytes(report.cache_memory)
        ));

        if !report.op_summaries.is_empty() {
            output.push_str("╠══════════════════════════════════════════════════════════════════════════════╣\n");
            output.push_str("║ OPERATIONS                                                                   ║\n");
            output.push_str(
                "╠════════════╤═══════════╤═══════════╤═══════════╤═══════════╤════════════════╣\n",
            );
            output.push_str(
                "║ Operation  │   Count   │  Avg(μs)  │  MEPS     │   GPU%    │   Cache%       ║\n",
            );
            output.push_str(
                "╠════════════╪═══════════╪═══════════╪═══════════╪═══════════╪════════════════╣\n",
            );

            for summary in &report.op_summaries {
                output.push_str(&format!(
                    "║ {:<10} │ {:>9} │ {:>9.2} │ {:>9.2} │ {:>8.1}% │ {:>12.1}%  ║\n",
                    summary.op.name(),
                    format_number(summary.count),
                    summary.avg_time_us,
                    summary.throughput_meps,
                    summary.gpu_pct,
                    summary.cache_hit_pct,
                ));
            }
        }

        output.push_str(
            "╚══════════════════════════════════════════════════════════════════════════════╝\n",
        );
        output
    }

    /// Reset all statistics
    pub fn reset(&self) {
        for stats in self.op_stats.values() {
            stats.count.store(0, Ordering::Relaxed);
            stats.total_time_ns.store(0, Ordering::Relaxed);
            stats.total_elements.store(0, Ordering::Relaxed);
            stats.total_bytes.store(0, Ordering::Relaxed);
            stats.gpu_executions.store(0, Ordering::Relaxed);
            stats.cpu_executions.store(0, Ordering::Relaxed);
            stats.cache_hits.store(0, Ordering::Relaxed);
        }
        self.dispatch_history.write().unwrap().clear();
        self.total_ops.store(0, Ordering::Relaxed);
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of operation statistics
#[derive(Debug, Clone)]
pub struct OpSummary {
    pub op: OpType,
    pub count: u64,
    pub avg_time_us: f64,
    pub throughput_meps: f64,
    pub bandwidth_gbps: f64,
    pub gpu_pct: f64,
    pub cache_hit_pct: f64,
}

/// Complete statistics report
#[derive(Debug, Clone)]
pub struct StatsReport {
    pub backend: String,
    pub duration: Duration,
    pub total_ops: u64,
    pub gpu_memory_current: u64,
    pub gpu_memory_peak: u64,
    pub cache_memory: u64,
    pub op_summaries: Vec<OpSummary>,
}

/// Format a number with thousands separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format bytes as human-readable
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_STATS: ExecutionStats = ExecutionStats::new();
}

/// Timer guard for measuring operation duration
pub struct OpTimer {
    op: OpType,
    start: Instant,
    elements: usize,
    bytes: usize,
    layer: ExecutionLayer,
    finished: bool,
}

impl OpTimer {
    pub fn new(op: OpType, elements: usize, layer: ExecutionLayer) -> Self {
        Self {
            op,
            start: Instant::now(),
            elements,
            bytes: elements * std::mem::size_of::<f32>(),
            layer,
            finished: false,
        }
    }

    pub fn with_bytes(op: OpType, elements: usize, bytes: usize, layer: ExecutionLayer) -> Self {
        Self {
            op,
            start: Instant::now(),
            elements,
            bytes,
            layer,
            finished: false,
        }
    }

    pub fn finish(mut self) {
        self.finished = true;
        GLOBAL_STATS.record_op(
            self.op,
            self.start.elapsed(),
            self.elements,
            self.bytes,
            self.layer,
        );
    }
}

impl Drop for OpTimer {
    fn drop(&mut self) {
        // Only record on drop if not explicitly finished
        if !self.finished {
            GLOBAL_STATS.record_op(
                self.op,
                self.start.elapsed(),
                self.elements,
                self.bytes,
                self.layer,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_recording() {
        let stats = ExecutionStats::new();
        stats.record_op(
            OpType::Add,
            Duration::from_micros(100),
            1000,
            4000,
            ExecutionLayer::GpuCompute,
        );
        stats.record_op(
            OpType::Add,
            Duration::from_micros(200),
            2000,
            8000,
            ExecutionLayer::Cpu,
        );

        let op_stats = stats.get_op_stats(OpType::Add).unwrap();
        assert_eq!(op_stats.count.load(Ordering::Relaxed), 2);
        assert_eq!(op_stats.gpu_executions.load(Ordering::Relaxed), 1);
        assert_eq!(op_stats.cpu_executions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_report_generation() {
        let stats = ExecutionStats::new();
        stats.set_backend("Test Backend");
        stats.record_op(
            OpType::MatMul,
            Duration::from_millis(10),
            1_000_000,
            4_000_000,
            ExecutionLayer::GpuCompute,
        );

        let report = stats.format_report();
        assert!(report.contains("Test Backend"));
        assert!(report.contains("MatMul"));
    }
}
