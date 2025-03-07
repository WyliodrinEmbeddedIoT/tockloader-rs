// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

mod action;
pub use action::Action;

mod state;
pub use state::{AppData, BoardConnectionStatus, State};

pub mod state_store_process;
pub use self::state_store_process::StateStore;
