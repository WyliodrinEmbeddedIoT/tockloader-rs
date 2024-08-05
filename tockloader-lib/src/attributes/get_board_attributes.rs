// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use probe_rs::{Core, MemoryInterface};

use super::hardware_attributes::HardwareAttributes;

pub(crate) fn get_bootloader_version(board_core: &mut Core, attributes: &mut HardwareAttributes) {
    let address = 0x40E;

    let mut buf = [0u8; 8];

    let _ = board_core.read_8(address, &mut buf);

    let decoder = utf8_decode::Decoder::new(buf.iter().cloned());

    let mut string = String::new();
    for n in decoder {
        string.push(n.expect("Error decoding bootloader version"));
    }

    let string = string.trim_matches(char::from(0));

    attributes.bootloader_version = Some(string.to_owned());
}

pub(crate) fn get_board_attributes(board_core: &mut Core, attributes: &mut HardwareAttributes) {
    let address = 0x600;
    let mut buf = [0u8; 64 * 16];
    let _ = board_core.read(address, &mut buf);

    let mut data = buf.chunks(64);

    for index_data in 0..data.len() {
        let step = match data.next() {
            Some(data) => data,
            None => break,
        };

        let step_option = decode_attribute(step);

        if step_option.is_none() {
            continue;
        }

        let step_hashmap = step_option.unwrap();

        match index_data {
            0 => {
                attributes.board = Some(step_hashmap[1].to_string());
            }
            1 => {
                attributes.arch = Some(step_hashmap[1].to_string());
            }
            2 => {
                attributes.appaddr = Some(step_hashmap[1].to_string());
            }
            3 => {
                attributes.boothash = Some(step_hashmap[1].to_string());
            }
            _ => panic!("Board data not found!"),
        }
    }
}

fn decode_attribute(step: &[u8]) -> Option<[String; 2]> {
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
    Some([key, value])
}

pub(crate) fn get_appaddr(board_core: &mut Core) -> Option<u64> {
    let address = 0x600;
    let mut buf = [0u8; 64 * 16];
    let _ = board_core.read(address, &mut buf);

    let mut data = buf.chunks(64);

    for _ in 0..data.len() {
        let step = match data.next() {
            Some(data) => data,
            None => break,
        };

        let step_option = decode_attribute(step);

        if step_option.is_none() {
            continue;
        }

        let step_hashmap = step_option.unwrap();

        if step_hashmap[0] == "appaddr" {
            return Some(
                u64::from_str_radix(step_hashmap[1].to_string().trim_start_matches("0x"), 16)
                    .unwrap(),
            );
        }
    }
    None
}
