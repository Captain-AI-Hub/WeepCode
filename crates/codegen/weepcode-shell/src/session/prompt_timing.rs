//! Per-turn prompt latency measurement.
//!
//! Implementation lives in `weepcode-telemetry::prompt_timing`. This shim
//! keeps `crate::session::prompt_timing::PromptTiming` resolving at the
//! original path so callers don't need to change imports.

pub(crate) use weepcode_telemetry::prompt_timing::PromptTiming;
