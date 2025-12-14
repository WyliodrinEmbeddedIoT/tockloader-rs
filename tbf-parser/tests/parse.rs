use tbf_parser::parse::*;
use tbf_parser::types::{TbfFooterV2Credentials, TbfFooterV2CredentialsType};

#[test]
fn simple_tbf() {
    let buffer = include_bytes!("./flashes/simple.dat").to_vec();

    let (ver, header_len, whole_len) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();
    assert_eq!(ver, 2);
    assert_eq!(header_len, 52);
    assert_eq!(whole_len, 8192);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    dbg!(&header);
    assert!(header.enabled());
    assert_eq!(header.get_minimum_app_ram_size(), 4848);
    assert_eq!(header.get_init_function_offset(), 41);
    assert_eq!(header.get_protected_trailer_size(), 0);
    assert_eq!(header.get_application_flags(), 1);
    assert_eq!(header.get_package_name().unwrap(), "_heart");
    assert_eq!(header.get_kernel_version().unwrap(), (2, 0));
}

#[test]
fn footer_sha256() {
    let buffer: Vec<u8> = include_bytes!("./flashes/footerSHA256.dat").to_vec();

    let (ver, header_len, whole_len) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();
    assert_eq!(ver, 2);
    assert_eq!(header_len, 76);
    assert_eq!(whole_len, 8192);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    dbg!(&header);
    assert!(header.enabled());
    assert_eq!(header.get_minimum_app_ram_size(), 4848);
    assert_eq!(header.get_init_function_offset(), 41);
    assert_eq!(header.get_protected_trailer_size(), 0);
    assert_eq!(header.get_application_flags(), 1);
    assert_eq!(header.get_package_name().unwrap(), "_heart");
    assert_eq!(header.get_kernel_version().unwrap(), (2, 0));
    let binary_offset = header.get_binary_end() as usize;
    assert_eq!(binary_offset, 5836);

    let (footer, footer_size) = parse_tbf_footer(&buffer[binary_offset..]).unwrap();
    dbg!(footer);
    assert_eq!(footer_size, 36);
    let correct_sha256 = [
        214u8, 17, 81, 32, 51, 178, 249, 35, 161, 33, 109, 184, 195, 46, 238, 158, 141, 54, 63, 94,
        60, 245, 50, 228, 239, 107, 231, 127, 220, 158, 77, 160,
    ];
    if let TbfFooterV2Credentials::SHA256(creds) = footer {
        assert_eq!(
            creds.get_format().unwrap(),
            TbfFooterV2CredentialsType::SHA256
        );
        assert_eq!(creds.get_hash(), &correct_sha256);
    } else {
        panic!("Footer is not of type SHA256!");
    }

    let second_footer_offset = binary_offset + footer_size as usize + 4;
    let (footer, footer_size) = parse_tbf_footer(&buffer[second_footer_offset..]).unwrap();
    dbg!(footer);
    assert_eq!(footer_size, 2312);
    if let TbfFooterV2Credentials::Reserved(padding) = footer {
        assert_eq!(padding, 2312);
    } else {
        panic!("Footer is not of type 'Reserved'!");
    }
}

#[test]
fn footer_rsa4096() {
    let buffer: Vec<u8> = include_bytes!("./flashes/footerRSA4096.dat").to_vec();

    let (ver, header_len, whole_len) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();
    assert_eq!(ver, 2);
    assert_eq!(header_len, 76);
    assert_eq!(whole_len, 4096);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    dbg!(&header);
    assert!(header.enabled());
    assert_eq!(header.get_minimum_app_ram_size(), 4612);
    assert_eq!(header.get_init_function_offset(), 41);
    assert_eq!(header.get_protected_trailer_size(), 0);
    assert_eq!(header.get_application_flags(), 1);
    assert_eq!(header.get_package_name().unwrap(), "c_hello");
    assert_eq!(header.get_kernel_version().unwrap(), (2, 0));
    let binary_offset = header.get_binary_end() as usize;
    assert_eq!(binary_offset, 1168);

    let (footer, footer_size) = parse_tbf_footer(&buffer[binary_offset..]).unwrap();
    dbg!(footer);
    assert_eq!(footer_size, 1028);
    let correct_key = include_bytes!("./flashes/RSA4096.key");
    let correct_signature = include_bytes!("./flashes/RSA4096.sig");
    if let TbfFooterV2Credentials::Rsa4096Key(creds) = footer {
        assert_eq!(
            creds.get_format().unwrap(),
            TbfFooterV2CredentialsType::Rsa4096Key
        );
        assert_eq!(creds.get_public_key(), correct_key);
        assert_eq!(creds.get_signature(), correct_signature);
    } else {
        panic!("Footer is not of type SHA256!");
    }

    let second_footer_offset = binary_offset + footer_size as usize + 4;
    let (footer, footer_size) = parse_tbf_footer(&buffer[second_footer_offset..]).unwrap();
    dbg!(footer);
    assert_eq!(footer_size, 1892);
    if let TbfFooterV2Credentials::Reserved(padding) = footer {
        assert_eq!(padding, 1892);
    } else {
        panic!("Footer is not of type 'Reserved'!");
    }
}

#[test]
fn shortid_valid() {
    let buffer: Vec<u8> = vec![
        0x02, 0x00, 0x2c, 0x00, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6a, 0x70, 0x41,
        0x73, 0x03, 0x00, 0x08, 0x00, 0x5f, 0x74, 0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x08, 0x00,
        0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x04, 0x00, 0xd2, 0x04, 0x00, 0x00,
    ];

    let (_ver, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    dbg!(&header);

    // Corrected: The expected ShortID is 1234 (0x4d2 in hex), not 0x12345678.
    let expected_short_id = core::num::NonZeroU32::new(1234);
    assert_eq!(header.get_fixed_short_id(), expected_short_id);
}

#[test]
fn shortid_invalid() {
    // The buffer generated by the previous step
    let invalid_short_buffer: Vec<u8> = vec![
        0x02, 0x00, 0x32, 0x00, 0xa5, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0xfc, 0xae,
        0x47, 0x03, 0x00, 0x08, 0x00, 0x5f, 0x74, 0x65, 0x73, 0x74, 0x00, 0x00, 0x00, 0x08, 0x00,
        0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x04, 0x00, 0xd2, 0x04, 0x00, 0x00,
    ];

    let result = parse_tbf_header(&invalid_short_buffer, 2);
    assert!(result.is_err());
    let error = result.unwrap_err();
    let debug_string = format!("{:?}", error);

    assert!(debug_string.contains("Checksum verification failed"));
}

#[test]
fn shortid_nonexistent() {
    // We can reuse the `simple.dat` artifact, as it should not have a ShortID TLV.
    let buffer = include_bytes!("./flashes/simple.dat").to_vec();

    let (_ver, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    dbg!(&header);

    // The core assertion: check that the ShortID is not present.
    assert_eq!(header.get_fixed_short_id(), None);
}

#[test]
fn ecdsa_nist_p256_smoke_test() {
    let tlv_type_credentials: u16 = 128; // 2 bytes
    let credential_format: u32 = TbfFooterV2CredentialsType::EcdsaNistP256 as u32; // 6
    let signature_r = [0xAA; 32];
    let signature_s = [0xBB; 32];

    // Length of payload: 4 (format) + 32 (r) + 32 (s) = 68
    let tlv_length: u16 = 68;

    let mut buffer = Vec::new();

    buffer.extend_from_slice(&tlv_type_credentials.to_le_bytes()); // 2 bytes
    buffer.extend_from_slice(&tlv_length.to_le_bytes()); // 2 bytes
    buffer.extend_from_slice(&credential_format.to_le_bytes()); // 4 bytes
    buffer.extend_from_slice(&signature_r); // 32 bytes
    buffer.extend_from_slice(&signature_s); // 32 bytes

    assert_eq!(buffer.len(), 4 + tlv_length as usize);

    match parse_tbf_footer(&buffer) {
        Ok((footer, returned_footer_size)) => {
            assert_eq!(returned_footer_size, tlv_length as u32);
            if let TbfFooterV2Credentials::EcdsaNistP256(ecdsa) = footer {
                assert_eq!(ecdsa.get_signature_r(), &signature_r);
                assert_eq!(ecdsa.get_signature_s(), &signature_s);
            } else {
                panic!("Footer is not of type EcdsaNistP256!");
            }
        }
        Err(e) => {
            panic!("Failed to parse footer: {:?}", e);
        }
    }
}
