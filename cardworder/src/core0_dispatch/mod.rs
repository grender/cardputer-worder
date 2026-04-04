//! Core 0 dispatch: type-safe action→result mapping with domain-split executor traits.
//!
//! Types are defined in `cardworder_core::types` and re-exported here for ESP use.

pub mod executor;

pub use cardworder_core::types::{
    Core0Action, Core0Result, Core0Executor,
    SystemExecutor, WifiExecutor, NtpExecutor, StorageExecutor,
    dispatch_action as dispatch,
};
