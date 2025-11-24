//! Code related to messages that clients and servers can pass to one another.

/// Maximum length of a message. Functions may return errors on receiving and panic on sending if
/// the message length exceeds this value
pub const MAX_MESSAGE_LENGTH: usize = u16::MAX as usize;

/// Maximum length of the message payload.
pub const MAX_MESSAGE_PAYLOAD_LENGTH: usize = MAX_MESSAGE_LENGTH - size_of::<MessageHeader>();

/// Size of the message header in bytes
pub const HEADER_SIZE: usize = size_of::<OpCode>() + size_of::<u16>();

/// The header of the message containing control fields.
///
/// # Serialization
///
/// If you are implementing the serialization mechanism for the header somewhere, **DON'T USE THE
/// SIZE OF THE STRUCT PROVIDED BY** [`std::mem::size_of<MessageHeader>()`]**!** The struct is
/// aligned, which means that the serialized bytes will be of a different length.
#[allow(missing_docs)] // the fields are painfully obvious
pub struct MessageHeader {
    pub opcode: OpCode,
    pub length: u16,
}

impl MessageHeader {
    /// Serialize the message header into bytes
    pub const fn serialize(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0; HEADER_SIZE];

        let (opcode, length) = buf.split_at_mut(size_of::<OpCode>());
        // yes, conversion to big endian is useless here, but if the opcode type changes sometime
        // later this will be needed
        opcode[0] = (self.opcode as u8).to_be();
        let length_bytes = self.length.to_be_bytes();
        length.copy_from_slice(&length_bytes);

        buf
    }

    /// Parse the header from a provided buffer
    pub fn deserialize(buf: &[u8; HEADER_SIZE]) -> crate::Result<Self> {
        let (opcode_buf, length_buf) = buf.split_at(size_of::<OpCode>());
        // TODO: check if this behaves right on little endian
        let opcode = if cfg!(target_endian = "big") {
            opcode_buf[0]
        } else {
            opcode_buf[0].to_le()
        };
        let opcode = OpCode::from_repr(opcode).ok_or(crate::Error::UnknownMessage)?;

        // TODO: also check if this behaves right
        let mut length: u16 = 0;
        length += <u8 as std::convert::Into<u16>>::into(length_buf[0] << 1);
        length += <u8 as std::convert::Into<u16>>::into(length_buf[1]);

        Ok(Self { opcode, length })
    }
}

#[cfg(test)]
mod header_tests {
    use std::io::Read;

    use crate::message::{HEADER_SIZE, MessageHeader, OpCode};

    #[test]
    fn serialize_header() {
        let header = MessageHeader {
            opcode: OpCode::StartMessage,
            length: 69,
        };

        let serialized = header.serialize();

        assert_eq!(serialized[0], (OpCode::StartMessage as u8).to_be());
        let length_offset: usize = 1;
        assert_eq!(
            &serialized[length_offset..(size_of::<u16>() + length_offset)],
            69_u16.to_be_bytes()
        );
    }

    #[test]
    fn deserialize_header() -> crate::Result<()> {
        const OPCODE: OpCode = OpCode::StartMessage;
        const LENGTH: u16 = 50;

        let mut header_buf = [0; HEADER_SIZE];

        let (opcode_field, length_field) = header_buf.split_at_mut(size_of::<OpCode>());
        opcode_field[0] = OPCODE as u8;
        length_field.copy_from_slice(&LENGTH.to_be_bytes());

        let header = MessageHeader::deserialize(&header_buf)?;
        assert_eq!(header.opcode, OPCODE);
        assert_eq!(header.length, LENGTH);

        Ok(())
    }
}

/// The different message opcode constants
///
/// # Stability
///
/// Don't rely on this being stable; this might dissappear at any time and I am actively looking
/// for options on how to move this closer to its usage.
#[allow(missing_docs)]
#[derive(strum::FromRepr, Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    StartMessage = 0,
}

/// A message that can serialize itself into a byte buffer as well as deserialize itself from one
pub trait Message {
    /// This needs to take in `&self` in order to be dyn compatible.
    fn opcode(&self) -> OpCode;
    /// Serialize message data into bytes
    fn serialize_data(&self) -> crate::Result<Box<[u8]>>;

    /// Serialize the message into bytes ready to be sent over the network. In contrast to
    /// [`Message::serialize_data`], this function is provided for messages by default and should
    /// almost never be overridden. If you need to override this function, you are either very
    /// cool and we should be friends, or you're doing something very, very wrong. Or both.
    fn serialize(&self) -> crate::Result<Box<[u8]>> {
        let data = self.serialize_data()?;
        if data.len() > MAX_MESSAGE_PAYLOAD_LENGTH {
            return Err(crate::Error::MesssageTooBig);
        }

        let header = MessageHeader {
            opcode: self.opcode(),
            // we should be fine here since we checked if data length is within the u16 range above
            length: data
                .len()
                .try_into()
                .expect("data length is outside of u16 range"),
        };
        let header_bytes = header.serialize();

        let mut buf = vec![];
        buf.reserve(header_bytes.len() + data.len());

        Ok(Box::from(buf.as_slice()))
    }
}

/// A message that tells the receiver that it should start a container
#[derive(Debug, PartialEq, Eq)]
pub struct StartMessage {
    /// The name of the resource/container to be started
    pub resource_name: String,
}

impl Message for StartMessage {
    fn opcode(&self) -> OpCode {
        OpCode::StartMessage
    }

    fn serialize_data(&self) -> crate::Result<Box<[u8]>> {
        let mut vec = vec![];

        let bytes = self.resource_name.as_bytes();
        vec.reserve(bytes.len());
        vec[0..bytes.len() as usize].copy_from_slice(bytes);

        // TODO: this checks the length only after serializing the data. make the check happen
        // before
        if vec.len() > MAX_MESSAGE_PAYLOAD_LENGTH {
            return Err(crate::Error::MesssageTooBig);
        }
        return Ok(Box::from(vec.as_slice()));
    }
}
