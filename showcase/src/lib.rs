//! One small activation per thing the IR can say. Each is a module that
//! compiles as ordinary Rust; `#[plexus::activation(describe)]` gives it its
//! `IR`, `WIT` and `CLIENT_TS`. Each IR is kept in `ir/` (see the README).
//! None is served.
#![allow(dead_code)]

pub mod f01_record;
pub mod f02_enum_and_variant;
pub mod f03_alias_and_limits;
pub mod f04_collections;
pub mod f05_default_and_deprecated;
pub mod f06_fails;
pub mod f07_stream_out;
pub mod f08_stream_in;
pub mod f09_progress;
pub mod f10_caller;
pub mod f11_grants_and_roles;
pub mod f12_children;
pub mod f13_async;
pub mod f14_four_ways_to_hold;
pub mod f15_wire_names;
pub mod compare_set_status;
