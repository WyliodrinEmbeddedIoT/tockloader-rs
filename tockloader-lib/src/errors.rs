// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TockloaderError {
    #[error("Serial connection error: {0}")]
    Serial(#[from] SerialError),

    #[error("Probe connection error: {0}")]
    Probe(#[from] ProbeError),

    #[error("TAB file error: {0}")]
    Tab(#[from] TabError),

    #[error("Tock OS error: {0}")]
    Tock(#[from] TockError),

    #[error("Internal tockloader error: {0}")]
    Internal(#[from] InternalError),
}

#[derive(Debug, Error)]
pub enum SerialError {
    #[error("Failed to interface in serial using tokio_serial: {0}")]
    TokioSerial(#[from] tokio_serial::Error),

    #[error("Failed to perform read/write operations on serial port: {0}")]
    IO(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("Failed to interact with probe: {0}")]
    Probe(#[from] probe_rs::probe::DebugProbeError),

    #[error("Communication with board failed: {0}")]
    Communication(#[from] probe_rs::Error),

    #[error("Failed to flash data: {0}")]
    Flashing(#[from] probe_rs::flashing::FlashError),
}

#[derive(Debug, Error)]
pub enum TabError {
    #[error("Failed to use tab from provided path: {0}")]
    Unusable(io::Error),

    #[error("Failed to parse metadata: {0}")]
    InvalidMetadata(toml::de::Error),

    #[error("No metadata.toml found")]
    NoMetadata,

    #[error("App data could not be parsed: {0:?}")]
    Parsing(tbf_parser::types::TbfParseError),

    #[error("No binary found for {0} architecture")]
    NoBinary(String),
}

#[derive(Debug, Error)]
pub enum TockError {
    #[error("Bootloader did not respond properly: {0}")]
    Bootloader(u8),

    #[error("Expected board attribute to be present: {0}")]
    MisconfiguredBoard(String),
}

#[derive(Debug, Error)]
pub enum InternalError {
    #[error("Operation failed due to board not being open.")]
    ConnectionNotOpen,

    #[error("Operation failed due to board not being in bootloader mode or not having a bootloader present.")]
    BootloaderNotPresent,
}
