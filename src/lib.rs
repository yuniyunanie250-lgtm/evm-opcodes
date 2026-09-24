//! EVM opcode metadata, and a cost estimate for a bytecode blob.
//!
//! Useful for quickly answering "how much does this cost" and "which opcodes
//! does this contract actually use" without booting a node.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Op {
    pub code: u8,
    pub name: &'static str,
    /// Base gas cost. Dynamic costs (memory, storage, calls) are not modelled.
    pub gas: u32,
    pub pops: u8,
    pub pushes: u8,
    /// True when this is a PUSH with an immediate operand.
    pub immediate: u8,
}

macro_rules! ops {
    ($($code:expr => ($name:expr, $gas:expr, $pops:expr, $pushes:expr, $imm:expr)),* $(,)?) => {
        &[$(Op { code: $code, name: $name, gas: $gas, pops: $pops, pushes: $pushes, immediate: $imm }),*]
    };
}

pub const TABLE: &[Op] = ops![
    0x00 => ("STOP", 0, 0, 0, 0),
    0x01 => ("ADD", 3, 2, 1, 0),
    0x02 => ("MUL", 5, 2, 1, 0),
    0x03 => ("SUB", 3, 2, 1, 0),
    0x04 => ("DIV", 5, 2, 1, 0),
    0x06 => ("MOD", 5, 2, 1, 0),
    0x10 => ("LT", 3, 2, 1, 0),
    0x14 => ("EQ", 3, 2, 1, 0),
    0x15 => ("ISZERO", 3, 1, 1, 0),
    0x16 => ("AND", 3, 2, 1, 0),
    0x17 => ("OR", 3, 2, 1, 0),
    0x19 => ("NOT", 3, 1, 1, 0),
    0x20 => ("KECCAK256", 30, 2, 1, 0),
    0x30 => ("ADDRESS", 2, 0, 1, 0),
    0x31 => ("BALANCE", 100, 1, 1, 0),
    0x33 => ("CALLER", 2, 0, 1, 0),
    0x34 => ("CALLVALUE", 2, 0, 1, 0),
    0x35 => ("CALLDATALOAD", 3, 1, 1, 0),
    0x36 => ("CALLDATASIZE", 2, 0, 1, 0),
    0x37 => ("CALLDATACOPY", 3, 3, 0, 0),
    0x38 => ("CODESIZE", 2, 0, 1, 0),
    0x3d => ("RETURNDATASIZE", 2, 0, 1, 0),
    0x42 => ("TIMESTAMP", 2, 0, 1, 0),
    0x43 => ("NUMBER", 2, 0, 1, 0),
    0x50 => ("POP", 2, 1, 0, 0),
    0x51 => ("MLOAD", 3, 1, 1, 0),
    0x52 => ("MSTORE", 3, 2, 0, 0),
    0x54 => ("SLOAD", 100, 1, 1, 0),
    0x55 => ("SSTORE", 100, 2, 0, 0),
    0x56 => ("JUMP", 8, 1, 0, 0),
    0x57 => ("JUMPI", 10, 2, 0, 0),
    0x58 => ("PC", 2, 0, 1, 0),
    0x5b => ("JUMPDEST", 1, 0, 0, 0),
    0x60 => ("PUSH1", 3, 0, 1, 1),
    0x61 => ("PUSH2", 3, 0, 1, 2),
    0x62 => ("PUSH3", 3, 0, 1, 3),
    0x63 => ("PUSH4", 3, 0, 1, 4),
    0x69 => ("PUSH10", 3, 0, 1, 10),
    0x6a => ("PUSH11", 3, 0, 1, 11),
    0x7f => ("PUSH32", 3, 0, 1, 32),
    0x80 => ("DUP1", 3, 1, 2, 0),
    0x81 => ("DUP2", 3, 2, 3, 0),
    0x90 => ("SWAP1", 3, 2, 2, 0),
    0x91 => ("SWAP2", 3, 3, 3, 0),
    0xa0 => ("LOG0", 375, 2, 0, 0),
    0xa1 => ("LOG1", 750, 3, 0, 0),
    0xa2 => ("LOG2", 1125, 4, 0, 0),
    0xf1 => ("CALL", 100, 7, 1, 0),
    0xf2 => ("CALLCODE", 100, 7, 1, 0),
    0xf3 => ("RETURN", 0, 2, 0, 0),
    0xf4 => ("DELEGATECALL", 100, 6, 1, 0),
    0xfa => ("STATICCALL", 100, 6, 1, 0),
    0xfd => ("REVERT", 0, 2, 0, 0),
    0xfe => ("INVALID", 0, 0, 0, 0),
    0xff => ("SELFDESTRUCT", 5000, 1, 0, 0),
];

/// Look up an opcode. `None` for anything not in [`TABLE`], which includes most
/// of the `DUP`/`SWAP`/`PUSH` ranges: only the commonly seen members are listed.
pub fn lookup(code: u8) -> Option<&'static Op> {
    TABLE.iter().find(|op| op.code == code)
}

/// Bytecode as hex (with or without `0x`) to bytes.
pub fn parse_hex(input: &str) -> Result<Vec<u8>, String> {
    let s = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
        .unwrap_or(input);
    if s.len() % 2 != 0 {
        return Err("odd number of hex digits".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Estimate {
    /// Total of all base gas costs.
    pub base_gas: u64,
    /// Instructions seen.
    pub instructions: usize,
    /// Opcodes encountered that are not in the table.
    pub unknown: usize,
    /// Instruction offset of every JUMPDEST.
    pub jumpdests: Vec<usize>,
}

/// Walk a bytecode blob, summing base gas and skipping PUSH immediates.
///
/// This is a static estimate: storage and memory costs depend on runtime values
/// and are not included, so treat the number as a floor.
pub fn estimate(bytecode: &[u8]) -> Estimate {
    let mut out = Estimate::default();
    let mut i = 0;
    while i < bytecode.len() {
        let code = bytecode[i];
        out.instructions += 1;
        match lookup(code) {
            Some(op) => {
                out.base_gas += op.gas as u64;
                if op.name == "JUMPDEST" {
                    out.jumpdests.push(i);
                }
                i += 1 + op.immediate as usize;
            }
            None => {
                out.unknown += 1;
                i += 1;
            }
        }
    }
    out
}
