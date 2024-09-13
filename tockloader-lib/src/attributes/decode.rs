// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

// Structure used to hold the data of the attributes region at the 0x600-0x9FF range
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

// Function used to decode 64 byte chunks from the attributes region at the 0x600-0x9FF range
// The first 8 bytes represent the key of the attribute,
// while the rest represent the stored data.
// The key is utf-8 decoded and converted into a String.
// Then we check the vlen value at the 9th byte,
// representing the bytewise length of the value from the attribute.
// Given that the chunks contain 64 bytes the value can not be larger than 55.
// We return None if the data is larger than the possible length or if the data does not exist for that particular attribute.
// Afterwards we decode the bytes of the value and turn them into a String.
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

// Function used to decode utf-8 encoded bytes and return them as Strings
// Specifically used in the read_system_attributes fn from the system_attributes.rs
// to decode the bytes of the sentinel kernel attribute.
pub(crate) fn bytes_to_string(raw: &[u8]) -> String {
    let decoder = utf8_decode::Decoder::new(raw.iter().cloned());

    let mut string = String::new();
    for n in decoder {
        string.push(n.expect("Error getting key for attributes."));
    }
    string
}
