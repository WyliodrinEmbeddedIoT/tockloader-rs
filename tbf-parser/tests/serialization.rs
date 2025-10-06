use tbf_parser::parse::*;
use tbf_parser::types::TbfHeader;

// Serialization

#[test]
fn serialize_identical_with_original() {
    let buffer: Vec<u8> = include_bytes!("./flashes/simple.dat").to_vec();

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
fn flags_modifications() {
    let mut buffer = include_bytes!("./flashes/footerSHA256.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
    header
        .set_sticky(true, &buffer[0..header_len as usize])
        .unwrap();
    // Set sticky without parsing
    assert!(header.sticky());
    // Unset
    header
        .set_sticky(false, &buffer[0..header_len as usize])
        .unwrap();

    // Disable
    header
        .set_enabled(false, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let reparsed = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(!reparsed.enabled());

    // Enable
    let mut header = reparsed;
    header
        .set_enabled(true, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let reparsed = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(reparsed.enabled());
}

#[test]
fn padding_header_set_flags() {
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
fn fields_preserved() {
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
fn multiple_flags_set() {
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

// Checksum //
#[test]
fn checksum() {
    // Try with empty_buffer
    let empty_buffer: Vec<u8> = vec![];
    let result = TbfHeader::compute_checksum(&empty_buffer, 0x00000006D);
    assert_eq!(result.unwrap(), 0x00000000);

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

// Complete use //
#[test]
fn serialization_multiple_checks() {
    let mut buffer = include_bytes!("./flashes/footerRSA4096.dat").to_vec();
    let (_, header_len, _) = parse_tbf_header_lengths(&buffer[0..8].try_into().unwrap())
        .ok()
        .unwrap();

    let mut header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());

    // Disable
    header
        .set_enabled(false, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(!header.enabled());

    // Enable and set sticky
    let mut header = header;
    header
        .set_enabled(true, &buffer[0..header_len as usize])
        .unwrap();
    header
        .set_sticky(true, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
    assert!(header.sticky());

    // Disable sticky with high bits
    let flags = 0xD6D6FFF1;
    let mut header = header;
    header
        .set_flags(flags, &buffer[0..header_len as usize])
        .unwrap();
    let serialized = header.serialize().unwrap();
    buffer[0..16].copy_from_slice(&serialized);

    let header = parse_tbf_header(&buffer[0..header_len as usize], 2).unwrap();
    assert!(header.enabled());
    assert!(!header.sticky());
    let flags_buffer = u32::from_le_bytes(buffer[8..12].try_into().unwrap());
    assert_eq!(flags_buffer, flags);
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
