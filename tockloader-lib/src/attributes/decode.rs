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

// TODO: explain what is happening here

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

// Used to decode, using the utf-8 standard, the 8 bytes
// that represent the key of the key-value pairs
// from the Attributes section of the Memory Layout
// Information taken from: [The Tock Book](https://book.tockos.org/doc/memory_layout.html?search=0x40000)
pub(crate) fn bytes_to_string(raw: &[u8]) -> String {
    let decoder = utf8_decode::Decoder::new(raw.iter().cloned());

    let mut string = String::new();
    for n in decoder {
        string.push(n.expect("Error getting key for attributes."));
    }
    string
}
