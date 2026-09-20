//! Deterministic simulation core.
//!
//! Everything under this module runs identically on every peer given the
//! same [`command::Command`] stream and initial state. Non-deterministic
//! types (like `std::collections::HashMap`) are forbidden here — use
//! `BTreeMap` or `Vec` instead.

pub mod collision;
pub mod command;
pub mod markers;
pub mod movement;
pub mod scheduler;
pub mod world;

pub use command::{CommandLog, PlayerCommand, TICK_DELAY_BEFORE_COMMAND_TAKES_EFFECT, Tick};
pub use world::{Sim, UNIT_PICK_RADIUS};
