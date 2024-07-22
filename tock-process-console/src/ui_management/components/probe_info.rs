// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

#[derive(Debug)]
pub struct ProbeInfo {
    pub(crate) port: usize,
    pub(crate) port_name: String,
    pub(crate) port_probe: String,
}

impl ProbeInfo {
    pub fn port(&self) -> &usize {
        &self.port
    }

    pub fn port_name(&self) -> &String {
        &self.port_name
    }

    pub fn port_probe(&self) -> &String {
        &self.port_probe
    }
}
