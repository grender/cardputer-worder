//! Safe wrappers for common ESP-IDF FFI calls.

/// Current time in microseconds from boot (monotonic).
pub fn now_us() -> u64 {
    // SAFETY: esp_timer_get_time is a pure read of a hardware timer,
    // takes no pointers, has no side effects, always safe to call.
    unsafe { esp_idf_svc::sys::esp_timer_get_time() as u64 }
}

/// Heap statistics.
pub struct HeapInfo {
    pub free_bytes: u32,
    pub total_bytes: u32,
    pub free_dma_bytes: u32,
    pub largest_block_bytes: u32,
}

/// Read current heap statistics.
pub fn heap_info() -> HeapInfo {
    // SAFETY: heap_caps_get_* are read-only queries, no pointer hazards.
    unsafe {
        HeapInfo {
            free_bytes: esp_idf_sys::heap_caps_get_free_size(esp_idf_sys::MALLOC_CAP_DEFAULT) as u32,
            total_bytes: esp_idf_sys::heap_caps_get_total_size(esp_idf_sys::MALLOC_CAP_DEFAULT) as u32,
            free_dma_bytes: esp_idf_sys::heap_caps_get_free_size(
                esp_idf_sys::MALLOC_CAP_DMA | esp_idf_sys::MALLOC_CAP_INTERNAL,
            ) as u32,
            largest_block_bytes: esp_idf_sys::heap_caps_get_largest_free_block(esp_idf_sys::MALLOC_CAP_DEFAULT) as u32,
        }
    }
}

/// Get current local time (hours, minutes, seconds).
pub fn local_time_hms() -> (i32, i32, i32) {
    // SAFETY: time() and localtime_r() are standard C functions,
    // writing to caller-owned stack variables. No aliasing issues.
    unsafe {
        let mut now_time: esp_idf_sys::time_t = 0;
        let mut tm: esp_idf_sys::tm = core::mem::zeroed();
        esp_idf_sys::time(&mut now_time);
        esp_idf_sys::localtime_r(&now_time, &mut tm);
        (tm.tm_hour, tm.tm_min, tm.tm_sec)
    }
}
