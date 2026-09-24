use evm_opcodes::{estimate, lookup, parse_hex};

#[test]
fn runtime_bytecode_from_solc_parses() {
    // PUSH1 0x80 PUSH1 0x40 MSTORE
    let code = parse_hex("0x6080604052").unwrap();
    let e = estimate(&code);
    assert_eq!(e.instructions, 3);
    assert_eq!(e.unknown, 0);
    assert_eq!(e.base_gas, 3 + 3 + 3);
}

#[test]
fn push_immediates_are_skipped_not_executed() {
    // PUSH1 0x5b would look like JUMPDEST if the operand were not skipped
    let code = parse_hex("0x605b").unwrap();
    let e = estimate(&code);
    assert_eq!(e.instructions, 1);
    assert!(
        e.jumpdests.is_empty(),
        "0x5b is an operand here, not an opcode"
    );
}

#[test]
fn jumpdests_are_recorded() {
    let code = parse_hex("0x5b60016000").unwrap();
    let e = estimate(&code);
    assert_eq!(e.jumpdests, vec![0]);
}

#[test]
fn unknown_opcodes_are_counted_not_panicked_on() {
    let code = parse_hex("0x0c").unwrap(); // 0x0c is not in the table
    let e = estimate(&code);
    assert_eq!(e.unknown, 1);
    assert_eq!(e.instructions, 1);
}

#[test]
fn push32_consumes_32_operand_bytes() {
    let mut hex = String::from("0x7f");
    hex.push_str(&"11".repeat(32));
    let code = parse_hex(&hex).unwrap();
    let e = estimate(&code);
    assert_eq!(e.instructions, 1);
    assert_eq!(code.len(), 33);
}

#[test]
fn lookup_returns_metadata_for_known_codes() {
    let op = lookup(0x55).unwrap();
    assert_eq!(op.name, "SSTORE");
    assert_eq!(op.gas, 100);
    assert_eq!(op.pops, 2);
    assert!(lookup(0x0c).is_none());
}

#[test]
fn hex_parsing_is_strict() {
    assert!(parse_hex("0xabc").is_err());
    assert!(parse_hex("0xzz").is_err());
    assert_eq!(parse_hex("6001").unwrap(), vec![0x60, 0x01]);
}
