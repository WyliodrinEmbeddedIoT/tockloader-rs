// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

///   Attributes are key-value pairs that describe hardware configuration, stored in a fixed 64-byte format:
///
/// 1. 8-byte null-padded UTF-8 key (e.g., "board__\0\0")
/// 2. 1-byte value length (1-55)
/// 3. Variable-length UTF-8 value (e.g., "nrf52840")
/// 4. Null padding to fill 64 bytes
///
/// These attributes are obtained by:
///
/// 1. Reading physical memory from 0x600-0x9FF (1024 bytes = 16 attribute slots)
/// 2. Decoding each 64-byte chunk into key-value pairs
/// 3. Storing valid pairs in this struct
///
/// Tock attributes examples:
///
/// 1. board: Hardware platform name
/// 2. arch: CPU architecture
/// 3. appaddr: Application memory start address
/// 4. boothash: Bootloader integrity checksum
///     
/// This structure used to hold the data of the attributes region at the 0x600-0x9FF range
#[derive(Debug)]
pub struct DecodedAttribute {
    pub key: String,
    pub value: String,
}

impl DecodedAttribute {
    pub(crate) fn new(decoded_key: String, decoded_value: String) -> DecodedAttribute {
        DecodedAttribute {
            key: decoded_key,
            value: decoded_value,
        }
    }
}
/// Function used to decode 64 byte chunks from the attributes region at the 0x600-0x9FF range
///
/// Each attribute follows this layout:
///
/// 1. Bytes 0–7: UTF-8 key string
/// 2. Byte 8: Value length (1–55)
/// 3. Bytes 9–63: UTF-8 value string
///
/// The function returns:
///
/// - `Some(DecodedAttribute)` containing the parsed key and value if valid
/// - `None` if:
/// - The value length is zero or exceeds 55 bytes (corrupt or uninitialized)
/// - The value contains invalid UTF-8 data
///
/// Panics
///
/// This function panics if the input step is less than 64 bytes.
/// The caller must ensure chunks are exactly 64 bytes.
pub(crate) fn decode_attribute(step: &[u8]) -> Option<DecodedAttribute> {
    let raw_key = &step[0..8];

    let decoder_key = utf8_decode::Decoder::new(raw_key.iter().cloned());

    let mut key = String::new();
    for n in decoder_key {
        key.push(n.expect("Error getting key for attributes."));
    }

    key = key.trim_end_matches('\0').to_string();
    let vlen = step[8];

    if vlen > 55 || vlen == 0 {
        return None;
    }
    let raw_value = &step[9..(9 + vlen as usize)];
    let decoder_value = utf8_decode::Decoder::new(raw_value.iter().cloned());

    let mut value = String::new();

    for n in decoder_value {
        value.push(n.expect("Error getting key for attributes."));
    }

    value = value.trim_end_matches('\0').to_string();
    Some(DecodedAttribute::new(key, value))
}

/// Specifically used in the read_system_attributes fn from the system_attributes.rs
///
/// to decode the bytes of the sentinel kernel attribute.
///
/// It's used to decode utf-8 encoded bytes and return them as Strings
pub(crate) fn bytes_to_string(raw: &[u8]) -> String {
    let decoder = utf8_decode::Decoder::new(raw.iter().cloned());

    let mut string = String::new();
    for n in decoder {
        string.push(n.expect("Error getting key for attributes."));
    }
    string
}
