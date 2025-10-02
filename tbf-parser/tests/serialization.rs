use tbf_parser::parse::*;
use tbf_parser::types::TbfHeader;

// Serialization

#[test]
fn simple_tbf() {
    let buffer: Vec<u8> = include_bytes!("./flashes/simple.dat").to_vec();

    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    let serialized = header.serialize().unwrap();

    // Check if serialize matches original buffer
    assert_eq!(&buffer[0..16], &serialized[..]);
}

#[test]
fn footer_sha256() {
    let buffer: Vec<u8> = include_bytes!("./flashes/footerSHA256.dat").to_vec();

    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    let serialized = header.serialize().unwrap();

    // Check if serialize matches original buffer
    assert_eq!(&buffer[0..16], &serialized[..]);
}

#[test]
fn footer_rsa4096() {
    let buffer: Vec<u8> = include_bytes!("./flashes/footerRSA4096.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    let serialized = header.serialize().unwrap();

    // Check if serialize matches original buffer
    assert_eq!(&buffer[0..16], &serialized[..]);
}

// Flag modifications
#[test]
fn disable_simple_tbf() {
    let mut buffer: Vec<u8> = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();

    assert!(header.enabled());
    assert_eq!(header.get_package_name().unwrap(), "_heart");

    // Set flag to 0 to disable the app
    header
        .set_flags(0x00000000, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    // Parse again and check if disable
    let reparsed = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(!reparsed.enabled());
    assert_eq!(reparsed.get_package_name().unwrap(), "_heart");
    assert_eq!(reparsed.get_minimum_app_ram_size(), 4848);
    assert_eq!(reparsed.get_init_function_offset(), 41 + header_len as u32);
    assert_eq!(reparsed.get_protected_size(), header_len as u32);
    assert_eq!(reparsed.get_kernel_version().unwrap(), (2, 0));
}

#[test]
fn enable_disable_footer_sha256() {
    let mut buffer = include_bytes!("./flashes/footerSHA256.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());

    // Disable
    header
        .set_flags(0x00000000, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let reparsed = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(!reparsed.enabled());

    // Check
    let mut header = reparsed;
    header
        .set_flags(0x00000001, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let reparsed = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(reparsed.enabled());
}

#[test]
fn padding_header() {
    let buffer = vec![
        0x02, 0x00, 0x10, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x10,
        0x00,
    ];
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();
    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();

    assert!(!header.enabled());
    assert!(!header.is_app());
    let serialized = header.serialize().unwrap();
    assert_eq!(&serialized[..], &buffer[0..16]);
}

#[test]
fn padding_header2() {
    let buffer = vec![
        0x02, 0x00, 0x10, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x00, 0x10,
        0x00,
    ];

    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();
    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();

    assert!(!header.is_app());

    let serialized = header.serialize();
    assert!(serialized.is_ok());

    assert!(header.set_flags(0x06000001, &buffer).is_ok());

    let serialized = header.serialize().unwrap();
    let flags = u32::from_le_bytes(serialized[8..12].try_into().unwrap());
    assert_eq!(flags, 0x06000001);
}

#[test]
fn fields() {
    let buffer = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();

    let header_size = header.header_size();
    let total_size = header.total_size();

    header
        .set_flags(0x00000003, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();

    let _version = u16::from_le_bytes(serialized[0..2].try_into().unwrap());
    let _header_size = u16::from_le_bytes(serialized[2..4].try_into().unwrap());
    let _total_size = u32::from_le_bytes(serialized[4..8].try_into().unwrap());

    // Check other fields are unchanged
    assert_eq!(_version, 2);
    assert_eq!(header_size, _header_size);
    assert_eq!(total_size, _total_size);
}

#[test]
fn multiple_set() {
    let buffer = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();

    // Trying to set multiple times to check consistency
    for i in 1..21 {
        header
            .set_flags(i, &buffer[0..header_len as usize])
            .unwrap();
        assert_eq!(header.enabled(), i % 2 == 1);
    }
}

#[test]
fn no_parsing() {
    let buffer = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    header
        .set_flags(0x00000003, &buffer[0..header_len as usize])
        .unwrap();

    assert!(header.enabled());
    assert!(header.sticky());
}

#[test]
fn unset_set() {
    let buffer = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();

    let s1 = header.serialize().unwrap();
    header
        .set_flags(0x80000001, &buffer[0..header_len as usize])
        .unwrap();
    let s2 = header.serialize().unwrap();
    header
        .set_flags(0x00000001, &buffer[0..header_len as usize])
        .unwrap();
    let s3 = header.serialize().unwrap();

    assert_ne!(s1, s2);
    assert_eq!(s1, s3);
    assert_ne!(s2, s3);
}

// Checksum //
#[test]
fn checksum() {
    let mut buffer = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();

    // Test with array of multiple flags for checksum validation
    for flags in [0x000FABCD, 0x00000001, 0x00000002, 0x00000003] {
        header
            .set_flags(flags, &buffer[0..header_len as usize])
            .unwrap();
        let serialized = header.serialize().unwrap();
        buffer[0..16].copy_from_slice(&serialized);

        let result = parse_tbf_header(&buffer[0..header_len as usize], 2);
        assert!(result.is_ok(), "Checksum validation failed for {flags}");
    }
}

#[test]
fn checksum_footer_sha256() {
    let buffer = include_bytes!("./flashes/footerSHA256.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let slice = &buffer[0..header_len as usize];
    let flags = u32::from_le_bytes(slice[8..12].try_into().unwrap());

    // Compute checksum
    let computed = TbfHeader::compute_checksum(slice, flags).unwrap();
    let stored = u32::from_le_bytes(slice[12..16].try_into().unwrap());

    // Compared with the one stored
    assert_eq!(computed, stored);
}

#[test]
fn empty_buffer() {
    let empty_buffer: Vec<u8> = vec![];
    let result = TbfHeader::compute_checksum(&empty_buffer, 0x00000006D);
    assert_eq!(result.unwrap(), 0x00000000);
}

// Complete use //
#[test]
fn all_simple_tbf() {
    let mut buffer = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());

    // Disable
    header
        .set_flags(0x00000000, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(!header.enabled());

    // Enable
    let mut header = header;
    header
        .set_flags(0x00000001, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
}

#[test]
fn all_rsa4096() {
    let mut buffer = include_bytes!("./flashes/footerRSA4096.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
    assert_eq!(header.get_package_name().unwrap(), "c_hello");

    // Disable
    header
        .set_flags(0x00000000, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(!header.enabled());
    assert_eq!(header.get_package_name().unwrap(), "c_hello");

    // Enable and set sticky
    let mut header = header;
    header
        .set_flags(0x00000003, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
    assert!(header.sticky());
}

#[test]
fn all_together_high_bits() {
    let mut buffer = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
    assert_eq!(header.get_package_name().unwrap(), "_heart");

    header
        .set_flags(0xFFFF0001, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header2 = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header2.enabled());

    // Verify flags are well set
    let flags_buffer = u32::from_le_bytes(buffer[8..12].try_into().unwrap());
    assert_eq!(flags_buffer, 0xFFFF0001);

    // Set to sticky and disabled
    let mut header3 = header;
    header3
        .set_flags(0xFFFF0002, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header3.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(!header.enabled());
    assert!(header.sticky());

    // Enable sticky and change the high bits
    let flags = 0xD6D60003;
    let mut header = header;
    header
        .set_flags(flags, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
    assert!(header.sticky());

    let flags_buffer = u32::from_le_bytes(buffer[8..12].try_into().unwrap());
    assert_eq!(flags_buffer, flags);

    // Clear high bits
    let mut header = header;
    header
        .set_flags(0x00000001, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
    assert!(!header.sticky());

    let final_flags = u32::from_le_bytes(buffer[8..12].try_into().unwrap());
    assert_eq!(final_flags, 0x00000001);

    assert_eq!(header.get_package_name().unwrap(), "_heart");
    assert_eq!(header.get_kernel_version().unwrap(), (2, 0));
}

#[test]
fn corrupt() {
    let mut buffer = include_bytes!("./flashes/simple.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    // Corrupt the checksum manually and check for parsing error
    buffer[12] ^= 0x6D;

    let result = parse_tbf_header(&buffer[0..header_len as usize], 2);
    assert!(result.is_err());
}
