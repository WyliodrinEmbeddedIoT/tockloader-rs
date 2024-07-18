// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

pub mod input_box;
pub mod output_box;
pub mod probe_info;

pub use input_box::InputBox;
pub use output_box::OutputBox;
pub use probe_info::ProbeInfo;

mod component;
pub use component::{Component, ComponentRender};
