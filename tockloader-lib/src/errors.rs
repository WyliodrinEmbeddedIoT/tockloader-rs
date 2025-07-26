// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::io;
use thiserror::Error;

// Rule of thumb: for public-facing functions or API use `TockloaderError`. For
// crate-public/private functions you can use more specific errors.

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
    UnusableTab(io::Error),

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
    #[error("Bootloader returned an invalid header: {0} {1}")]
    BootloaderBadHeader(u8, u8),

    #[error("Bootloader command did not finish in time")]
    BootloaderTimeout,

    #[error("Application data could not be parsed due to malformed header: {0:?}")]
    InvalidAppTbfHeader(tbf_parser::types::TbfParseError),

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

impl From<tokio_serial::Error> for TockloaderError {
    fn from(value: tokio_serial::Error) -> Self {
        TockloaderError::Serial(value.into())
    }
}

impl From<probe_rs::Error> for TockloaderError {
    fn from(value: probe_rs::Error) -> Self {
        TockloaderError::Probe(value.into())
    }
}

impl From<probe_rs::flashing::FlashError> for TockloaderError {
    fn from(value: probe_rs::flashing::FlashError) -> Self {
        TockloaderError::Probe(value.into())
    }
}

impl From<probe_rs::probe::DebugProbeError> for TockloaderError {
    fn from(value: probe_rs::probe::DebugProbeError) -> Self {
        TockloaderError::Probe(value.into())
    }
}
