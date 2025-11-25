# Cower protocol

## Packets

Cower packets (from now on referred to only as packets) consist of two parts:

- **header** - right now 3 bytes long
- **payload** - `u16::MAX - HEADER_SIZE`

### Header

The header has two parts:

- `opcode` - `u8`
- `payload_length` - `u16::MAX - HEADER_SIZE`

The opcode specifies the type of message. Most of the `u8` range isn't used, and
meaning of unused discriminants can change at any time. On the other hand,
defined discriminants don't change meaning (or at least I try my best not to
change it).

Payload length is in bytes. A `u16` can hold values up to `65_535`, and the
header takes up 3 bytes, so the payload can be up to `65_532` bytes (or
approximately 64 KiB). This is pretty overkill for a protocol like this, but
it's only an upper bound.

### Payload

The meaning of payload data changes from opcode to opcode, so the details aren't
listed here. Instead, check out
[`message.rs`](https://github.com/stanekondrej/cower/blob/a90b743c99b528f438d034f0424c30ea6321c90d/cower-common/src/message.rs#L126)
for up-to-date definitions.

The payload CAN be empty, in which case `payload_length` will be set to `0`.
