// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use super::{app_attributes::AppAttributes, system_attributes::SystemAttributes};

// Struncture used for packaging
// all retreaved data from the board
// that tockloader-cli might want to access.
#[derive(Debug)]
pub struct GeneralAttributes {
    pub system: SystemAttributes,
    pub apps: Vec<AppAttributes>,
}

impl GeneralAttributes {
    pub(crate) fn new(
        system_attributes: SystemAttributes,
        apps_attributes: Vec<AppAttributes>,
    ) -> GeneralAttributes {
        GeneralAttributes {
            system: system_attributes,
            apps: apps_attributes,
        }
    }
}
