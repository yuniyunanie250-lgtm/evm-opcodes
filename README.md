# evm-opcodes

Opcode metadata for the EVM, plus a static base-gas estimate for a bytecode blob.

Two questions this answers without a node: *which opcodes does this contract
use*, and *what is the floor cost of running it*. Both come up when reviewing a
contract or sizing a deployment.

## Usage

```rust
use evm_opcodes::{estimate, lookup, parse_hex};

let op = lookup(0x55).unwrap();
assert_eq!(op.name, "SSTORE");
assert_eq!(op.pops, 2);

let code = parse_hex("0x6080604052").unwrap();
let e = estimate(&code);
assert_eq!(e.base_gas, 9);       // three PUSH1/MSTORE at 3 gas each
```

## The one subtlety that matters

**PUSH immediates are data, not code.** The operand of `PUSH1 0x5b` is the byte
`0x5b`, which happens to be `JUMPDEST`. A disassembler that walks byte by byte
without skipping operands reports a jump destination that does not exist, and
that error is how jump-target analysis goes wrong. `estimate` skips them, and
there is a test for exactly this case.

## Limits

- **Base gas only.** Storage, memory expansion, and call costs depend on runtime
  values and are not modelled, so the number is a floor, not a quote.
- **Partial table.** The commonly seen opcodes are listed. The full `DUP`,
  `SWAP` and `PUSH` ranges are not, and unknown codes are counted rather than
  guessed at.
- **Post-Cancun opcodes are not included** (`TLOAD`, `TSTORE`, `MCOPY`, `BLOBHASH`).

## Development

```bash
cargo test
```

## License

MIT
