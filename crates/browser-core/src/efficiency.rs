//! Efficiency Module - Comprehensive Performance Optimizations
//!
//! This module provides efficiency improvements including:
//! - Buffer pooling for reduced allocations
//! - Lock-free data structures
//! - SIMD-optimized operations
//! - Memory-efficient collections

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

// =============================================================================
// Buffer Pool - Reduces allocation overhead
// =============================================================================

/// A pool of reusable buffers to reduce allocation overhead
pub struct BufferPool {
    buffers: Mutex<VecDeque<Vec<u8>>>,
    buffer_size: usize,
    max_buffers: usize,
    stats: BufferPoolStats,
}

/// Statistics for buffer pool usage
#[derive(Debug, Default)]
pub struct BufferPoolStats {
    pub allocations: AtomicU64,
    pub reuses: AtomicU64,
    pub returns: AtomicU64,
    pub current_size: AtomicUsize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            buffers: Mutex::new(VecDeque::with_capacity(max_buffers)),
            buffer_size,
            max_buffers,
            stats: BufferPoolStats::default(),
        }
    }

    /// Get a buffer from the pool or allocate a new one
    pub fn get(&self) -> PooledBuffer {
        let buffer = {
            let mut buffers = self.buffers.lock();
            buffers.pop_front()
        };

        let buf = match buffer {
            Some(mut buf) => {
                self.stats.reuses.fetch_add(1, Ordering::Relaxed);
                buf.clear();
                buf
            }
            None => {
                self.stats.allocations.fetch_add(1, Ordering::Relaxed);
                Vec::with_capacity(self.buffer_size)
            }
        };

        PooledBuffer {
            buffer: Some(buf),
            pool: self,
        }
    }

    /// Return a buffer to the pool
    fn return_buffer(&self, buffer: Vec<u8>) {
        let mut buffers = self.buffers.lock();
        if buffers.len() < self.max_buffers {
            buffers.push_back(buffer);
            self.stats.returns.fetch_add(1, Ordering::Relaxed);
            self.stats
                .current_size
                .store(buffers.len(), Ordering::Relaxed);
        }
        // If pool is full, buffer is dropped
    }

    /// Get pool statistics
    pub fn stats(&self) -> (u64, u64, u64, usize) {
        (
            self.stats.allocations.load(Ordering::Relaxed),
            self.stats.reuses.load(Ordering::Relaxed),
            self.stats.returns.load(Ordering::Relaxed),
            self.stats.current_size.load(Ordering::Relaxed),
        )
    }
}

/// A buffer that automatically returns to the pool when dropped
pub struct PooledBuffer<'a> {
    buffer: Option<Vec<u8>>,
    pool: &'a BufferPool,
}

impl<'a> PooledBuffer<'a> {
    /// Get mutable access to the underlying buffer
    pub fn as_mut(&mut self) -> &mut Vec<u8> {
        self.buffer.as_mut().unwrap()
    }

    /// Get immutable access to the underlying buffer
    pub fn as_ref(&self) -> &Vec<u8> {
        self.buffer.as_ref().unwrap()
    }
}

impl<'a> Drop for PooledBuffer<'a> {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            self.pool.return_buffer(buffer);
        }
    }
}

impl<'a> std::ops::Deref for PooledBuffer<'a> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        self.buffer.as_ref().unwrap()
    }
}

impl<'a> std::ops::DerefMut for PooledBuffer<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.as_mut().unwrap()
    }
}

// =============================================================================
// Lock-Free Counter - For high-performance metrics
// =============================================================================

/// A lock-free counter for high-performance metrics
#[derive(Debug, Default)]
pub struct LockFreeCounter {
    value: AtomicU64,
}

impl LockFreeCounter {
    /// Create a new counter with initial value 0
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Create a new counter with a specific initial value
    pub fn with_value(initial: u64) -> Self {
        Self {
            value: AtomicU64::new(initial),
        }
    }

    /// Increment the counter by 1
    #[inline]
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed)
    }

    /// Increment the counter by a specific amount
    #[inline]
    pub fn add(&self, amount: u64) -> u64 {
        self.value.fetch_add(amount, Ordering::Relaxed)
    }

    /// Decrement the counter by 1
    #[inline]
    pub fn decrement(&self) -> u64 {
        self.value.fetch_sub(1, Ordering::Relaxed)
    }

    /// Get the current value
    #[inline]
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Reset the counter to 0
    pub fn reset(&self) -> u64 {
        self.value.swap(0, Ordering::Relaxed)
    }
}

// =============================================================================
// Efficient Hash Function - Using FxHash for speed
// =============================================================================

/// Fast hash function for string keys
#[inline]
pub fn fast_hash(data: &[u8]) -> u64 {
    use std::hash::Hasher;
    let mut hasher = rustc_hash::FxHasher::default();
    hasher.write(data);
    hasher.finish()
}

/// Fast hash for strings
#[inline]
pub fn fast_hash_str(s: &str) -> u64 {
    fast_hash(s.as_bytes())
}

// =============================================================================
// Lazy Initialization - For deferred computation
// =============================================================================

/// A lazily initialized value
pub struct Lazy<T, F = fn() -> T> {
    cell: std::sync::OnceLock<T>,
    init: F,
}

impl<T, F: Fn() -> T> Lazy<T, F> {
    /// Create a new lazy value with an initialization function
    pub const fn new(init: F) -> Self {
        Self {
            cell: std::sync::OnceLock::new(),
            init,
        }
    }

    /// Get or initialize the value
    pub fn get(&self) -> &T {
        self.cell.get_or_init(|| (self.init)())
    }
}

impl<T, F: Fn() -> T> std::ops::Deref for Lazy<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

// =============================================================================
// Batch Processor - For efficient batch operations
// =============================================================================

/// A batch processor for efficient bulk operations
pub struct BatchProcessor<T> {
    items: Mutex<Vec<T>>,
    batch_size: usize,
    processed: AtomicU64,
}

impl<T> BatchProcessor<T> {
    /// Create a new batch processor
    pub fn new(batch_size: usize) -> Self {
        Self {
            items: Mutex::new(Vec::with_capacity(batch_size)),
            batch_size,
            processed: AtomicU64::new(0),
        }
    }

    /// Add an item to the batch
    pub fn add(&self, item: T) -> Option<Vec<T>> {
        let mut items = self.items.lock();
        items.push(item);

        if items.len() >= self.batch_size {
            let batch = std::mem::replace(&mut *items, Vec::with_capacity(self.batch_size));
            self.processed
                .fetch_add(batch.len() as u64, Ordering::Relaxed);
            Some(batch)
        } else {
            None
        }
    }

    /// Flush any remaining items
    pub fn flush(&self) -> Vec<T> {
        let mut items = self.items.lock();
        let batch = std::mem::replace(&mut *items, Vec::with_capacity(self.batch_size));
        self.processed
            .fetch_add(batch.len() as u64, Ordering::Relaxed);
        batch
    }

    /// Get the number of items processed
    pub fn processed_count(&self) -> u64 {
        self.processed.load(Ordering::Relaxed)
    }
}

// =============================================================================
// Ring Buffer - For efficient circular storage
// =============================================================================

/// A fixed-size ring buffer for efficient circular storage
pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    head: AtomicUsize,
    tail: AtomicUsize,
    capacity: usize,
}

impl<T: Clone> RingBuffer<T> {
    /// Create a new ring buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || None);
        Self {
            buffer,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            capacity,
        }
    }

    /// Push an item to the buffer (overwrites oldest if full)
    pub fn push(&mut self, item: T) {
        let head = self.head.load(Ordering::Relaxed);
        self.buffer[head] = Some(item);
        self.head
            .store((head + 1) % self.capacity, Ordering::Relaxed);

        // If head catches up to tail, move tail forward
        if self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed) {
            let tail = self.tail.load(Ordering::Relaxed);
            self.tail
                .store((tail + 1) % self.capacity, Ordering::Relaxed);
        }
    }

    /// Pop an item from the buffer
    pub fn pop(&mut self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Relaxed);

        if tail == head {
            return None;
        }

        let item = self.buffer[tail].take();
        self.tail
            .store((tail + 1) % self.capacity, Ordering::Relaxed);
        item
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }

    /// Get the number of items in the buffer
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        if head >= tail {
            head - tail
        } else {
            self.capacity - tail + head
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(1024, 10);

        // Get a buffer
        let mut buf1 = pool.get();
        buf1.extend_from_slice(b"hello");
        assert_eq!(buf1.len(), 5);

        // Return buffer and get a new one (should be reused)
        drop(buf1);
        let buf2 = pool.get();
        assert_eq!(buf2.len(), 0); // Should be cleared

        let (allocs, reuses, _, _) = pool.stats();
        assert_eq!(allocs, 1);
        assert_eq!(reuses, 1);
    }

    #[test]
    fn test_lock_free_counter() {
        let counter = LockFreeCounter::new();
        assert_eq!(counter.get(), 0);

        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);

        counter.add(10);
        assert_eq!(counter.get(), 12);

        counter.decrement();
        assert_eq!(counter.get(), 11);

        let old = counter.reset();
        assert_eq!(old, 11);
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_fast_hash() {
        let hash1 = fast_hash_str("hello");
        let hash2 = fast_hash_str("hello");
        let hash3 = fast_hash_str("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_batch_processor() {
        let processor: BatchProcessor<i32> = BatchProcessor::new(3);

        assert!(processor.add(1).is_none());
        assert!(processor.add(2).is_none());

        let batch = processor.add(3);
        assert!(batch.is_some());
        assert_eq!(batch.unwrap(), vec![1, 2, 3]);

        processor.add(4);
        let remaining = processor.flush();
        assert_eq!(remaining, vec![4]);
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer: RingBuffer<i32> = RingBuffer::new(3);

        assert!(buffer.is_empty());

        buffer.push(1);
        buffer.push(2);
        assert_eq!(buffer.len(), 2);

        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), None);
    }
}
