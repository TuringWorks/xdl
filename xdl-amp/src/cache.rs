//! Buffer caching and memory management for XDL-AMP
//!
//! Provides intelligent caching of GPU buffers to reduce
//! memory allocation overhead and data transfer latency.

use crate::backend::{GpuBuffer, GpuDevice};
use crate::error::Result;
use crate::stats::{ExecutionLayer, OpType, GLOBAL_STATS};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Configuration for the buffer cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum total cache size in bytes
    pub max_size_bytes: usize,
    /// Maximum number of cached buffers
    pub max_buffers: usize,
    /// Time-to-live for cached buffers
    pub ttl: Duration,
    /// Minimum array size (elements) to cache
    pub min_cache_elements: usize,
    /// Enable result caching (memoization)
    pub enable_result_cache: bool,
    /// Maximum result cache entries
    pub max_result_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 512 * 1024 * 1024, // 512 MB
            max_buffers: 1000,
            ttl: Duration::from_secs(60),
            min_cache_elements: 1000,
            enable_result_cache: true,
            max_result_entries: 100,
        }
    }
}

/// Hash key for result caching
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct ResultKey {
    pub op: String,
    pub input_hash: u64,
    pub shape: Vec<usize>,
}

/// Cached result entry
struct CachedResult {
    data: Vec<u8>,
    last_access: Instant,
    access_count: u64,
}

/// Buffer pool for reusing GPU allocations
pub struct BufferPool {
    /// Free buffers by size bucket
    free_buffers: RwLock<HashMap<usize, Vec<Box<dyn GpuBuffer>>>>,
    /// Total bytes in pool
    total_bytes: std::sync::atomic::AtomicUsize,
    /// Config
    config: CacheConfig,
}

impl BufferPool {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            free_buffers: RwLock::new(HashMap::new()),
            total_bytes: std::sync::atomic::AtomicUsize::new(0),
            config,
        }
    }

    /// Get a buffer from the pool or allocate new
    pub fn get_or_allocate(
        &self,
        device: &Arc<dyn GpuDevice>,
        size: usize,
    ) -> Result<Box<dyn GpuBuffer>> {
        // Round up to nearest power of 2 for better reuse
        let bucket_size = size.next_power_of_two();

        // Try to get from pool
        {
            let mut pool = self.free_buffers.write().unwrap();
            if let Some(buffers) = pool.get_mut(&bucket_size) {
                if let Some(buffer) = buffers.pop() {
                    self.total_bytes.fetch_sub(bucket_size, std::sync::atomic::Ordering::Relaxed);
                    GLOBAL_STATS.record_cache_memory(-(bucket_size as i64));
                    return Ok(buffer);
                }
            }
        }

        // Allocate new buffer
        let timer_start = std::time::Instant::now();
        let buffer = device.create_buffer(bucket_size)?;
        GLOBAL_STATS.record_op(
            OpType::BufferAlloc,
            timer_start.elapsed(),
            1,
            bucket_size,
            ExecutionLayer::GpuMemory,
        );
        GLOBAL_STATS.record_gpu_alloc(bucket_size);

        Ok(buffer)
    }

    /// Return a buffer to the pool for reuse
    pub fn return_buffer(&self, buffer: Box<dyn GpuBuffer>) {
        let size = buffer.size();
        let bucket_size = size.next_power_of_two();

        let current_total = self.total_bytes.load(std::sync::atomic::Ordering::Relaxed);

        // Check if we have room in the pool
        if current_total + bucket_size > self.config.max_size_bytes {
            // Pool is full, just drop the buffer
            GLOBAL_STATS.record_gpu_free(size);
            return;
        }

        let mut pool = self.free_buffers.write().unwrap();
        let buffers = pool.entry(bucket_size).or_insert_with(Vec::new);

        if buffers.len() < 100 {
            // Keep up to 100 buffers per size bucket
            buffers.push(buffer);
            self.total_bytes.fetch_add(bucket_size, std::sync::atomic::Ordering::Relaxed);
            GLOBAL_STATS.record_cache_memory(bucket_size as i64);
        } else {
            GLOBAL_STATS.record_gpu_free(size);
        }
    }

    /// Clear the entire pool
    pub fn clear(&self) {
        let mut pool = self.free_buffers.write().unwrap();
        let total = self.total_bytes.swap(0, std::sync::atomic::Ordering::Relaxed);
        GLOBAL_STATS.record_cache_memory(-(total as i64));
        pool.clear();
    }

    /// Get current pool size in bytes
    pub fn size_bytes(&self) -> usize {
        self.total_bytes.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Result cache for memoizing operation results
pub struct ResultCache {
    cache: RwLock<HashMap<ResultKey, CachedResult>>,
    config: CacheConfig,
    total_bytes: std::sync::atomic::AtomicUsize,
}

impl ResultCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            config,
            total_bytes: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Get a cached result if available
    pub fn get(&self, key: &ResultKey) -> Option<Vec<u8>> {
        if !self.config.enable_result_cache {
            return None;
        }

        let mut cache = self.cache.write().unwrap();
        if let Some(entry) = cache.get_mut(key) {
            // Check TTL
            if entry.last_access.elapsed() > self.config.ttl {
                let size = entry.data.len();
                cache.remove(key);
                self.total_bytes.fetch_sub(size, std::sync::atomic::Ordering::Relaxed);
                GLOBAL_STATS.record_cache_memory(-(size as i64));
                return None;
            }

            entry.last_access = Instant::now();
            entry.access_count += 1;
            return Some(entry.data.clone());
        }
        None
    }

    /// Store a result in the cache
    pub fn put(&self, key: ResultKey, data: Vec<u8>) {
        if !self.config.enable_result_cache {
            return;
        }

        let data_size = data.len();
        let mut cache = self.cache.write().unwrap();

        // Evict if at capacity
        while cache.len() >= self.config.max_result_entries {
            // Find LRU entry
            let lru_key = cache
                .iter()
                .min_by_key(|(_, v)| v.last_access)
                .map(|(k, _)| k.clone());

            if let Some(key) = lru_key {
                if let Some(entry) = cache.remove(&key) {
                    let size = entry.data.len();
                    self.total_bytes.fetch_sub(size, std::sync::atomic::Ordering::Relaxed);
                    GLOBAL_STATS.record_cache_memory(-(size as i64));
                }
            } else {
                break;
            }
        }

        cache.insert(
            key,
            CachedResult {
                data,
                last_access: Instant::now(),
                access_count: 1,
            },
        );
        self.total_bytes.fetch_add(data_size, std::sync::atomic::Ordering::Relaxed);
        GLOBAL_STATS.record_cache_memory(data_size as i64);
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        let total = self.total_bytes.swap(0, std::sync::atomic::Ordering::Relaxed);
        GLOBAL_STATS.record_cache_memory(-(total as i64));
        cache.clear();
    }

    /// Get current cache size in bytes
    pub fn size_bytes(&self) -> usize {
        self.total_bytes.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Compute a hash for f32 array (for result caching)
pub fn hash_f32_array(data: &[f32]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    // Hash length
    data.len().hash(&mut hasher);

    // Sample hash (for performance on large arrays)
    let step = if data.len() > 1000 {
        data.len() / 100
    } else {
        1
    };

    for (i, &val) in data.iter().enumerate() {
        if i % step == 0 {
            val.to_bits().hash(&mut hasher);
        }
    }

    hasher.finish()
}

/// Unified cache manager
pub struct CacheManager {
    pub buffer_pool: BufferPool,
    pub result_cache: ResultCache,
    config: CacheConfig,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            buffer_pool: BufferPool::new(config.clone()),
            result_cache: ResultCache::new(config.clone()),
            config,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Clear all caches
    pub fn clear_all(&self) {
        self.buffer_pool.clear();
        self.result_cache.clear();
    }

    /// Get total cache memory usage
    pub fn total_memory(&self) -> usize {
        self.buffer_pool.size_bytes() + self.result_cache.size_bytes()
    }

    /// Get cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_f32_array() {
        let arr1 = vec![1.0f32, 2.0, 3.0];
        let arr2 = vec![1.0f32, 2.0, 3.0];
        let arr3 = vec![1.0f32, 2.0, 4.0];

        assert_eq!(hash_f32_array(&arr1), hash_f32_array(&arr2));
        assert_ne!(hash_f32_array(&arr1), hash_f32_array(&arr3));
    }

    #[test]
    fn test_result_cache() {
        let config = CacheConfig {
            enable_result_cache: true,
            max_result_entries: 10,
            ttl: Duration::from_secs(60),
            ..Default::default()
        };

        let cache = ResultCache::new(config);

        let key = ResultKey {
            op: "add".to_string(),
            input_hash: 12345,
            shape: vec![100],
        };

        let data = vec![1u8, 2, 3, 4];
        cache.put(key.clone(), data.clone());

        let result = cache.get(&key);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), data);
    }
}
