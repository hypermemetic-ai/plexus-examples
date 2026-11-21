//! One small activation per thing the IR can say. The macro provides
//! `IR`, `WIT` and `CLIENT_TS`. Each IR is kept in `ir/` (see the README).
//! The native media example serves the input-stream fixture.
#![allow(dead_code)]

pub mod f01_record;
pub mod f02_enum_and_variant;
pub mod f03_alias_and_limits;
pub mod f04_collections;
pub mod f05_default_and_deprecated;
pub mod f06_fails;
pub mod f07_stream_out;
#[cfg(not(target_arch = "wasm32"))]
pub mod f08_stream_in;
pub mod f09_progress;
pub mod f10_caller;
pub mod f11_grants_and_roles;
pub mod f12_children;
pub mod f13_async;
pub mod f14_four_ways_to_hold;
pub mod f15_wire_names;
pub mod compare_set_status;
