# Cower protocol

## Packets

Cower packets (from now on referred to only as packets) consist of two parts:

- **header** - right now 3 bytes long
- **payload** - `u16::MAX`

Therefore, packets can be up to `u16::MAX + 3` bytes long.

### Header

The header has two parts:

- `opcode` - `u8`
- `payload_length` - `u16`

The opcode specifies the type of message. Most of the `u8` range isn't used, and
meaning of unused discriminants can change at any time. On the other hand,
defined discriminants don't change meaning (or at least I try my best not to
change it).

Payload length is in bytes, so the payload can be up to `u16::MAX` bytes (or
approximately 64 KiB) long. This is pretty overkill for a protocol like this, but
it's only an upper bound.

### Payload

The meaning of payload data changes from opcode to opcode, so the details aren't
listed here. Instead, check out
[`message.rs`](https://github.com/stanekondrej/cower/blob/a90b743c99b528f438d034f0424c30ea6321c90d/cower-common/src/message.rs#L126)
for up-to-date definitions.

The payload CAN be empty, in which case `payload_length` will be set to `0`.
