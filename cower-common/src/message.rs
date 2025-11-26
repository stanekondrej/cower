//! Code related to messages that clients and servers can pass to one another.

/// Maximum length of a message payload
pub const MAX_MESSAGE_PAYLOAD_LENGTH: u16 = u16::MAX;

/// Size of the message header in bytes
// this doesn't take the size directly from `size_of::<MessageHeader>()` because alignment is
// something that (fortunately) doesn't apply to bytes sent over the network.
pub const HEADER_SIZE: u16 = (size_of::<OpCode>() + size_of::<u16>()) as u16;

/// Max message length, header length and payload length combined
pub const MAX_MESSAGE_LENGTH: usize = MAX_MESSAGE_PAYLOAD_LENGTH as usize + HEADER_SIZE as usize;

/// The different message opcode constants
///
/// # Stability
///
/// Don't rely on this specific enum being stable; this might dissappear at any time and I am
/// actively looking for options on how to move this closer to its usage.
///
/// However, feel free to rely on the stability of the discriminants themselves. I'll try not to
/// change them so that message passing ideally still works between minor version changes.
#[allow(missing_docs)]
#[derive(strum::FromRepr, Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    StartMessage = 0,
}

/// The header of the message containing control fields
///
/// # Serialization
///
/// If you are implementing the serialization mechanism for the header somewhere, **DON'T USE THE
/// SIZE OF THE STRUCT PROVIDED BY** [`std::mem::size_of<MessageHeader>()`]**!** The struct is
/// aligned, which means that the serialized bytes will be of a different length.
#[allow(missing_docs)] // the fields are painfully obvious
#[derive(Debug)]
pub struct MessageHeader {
    pub opcode: OpCode,
    pub length: u16,
}

impl MessageHeader {
    /// Serialize the message header into bytes
    pub fn serialize(&self) -> [u8; HEADER_SIZE as usize] {
        let mut buf = [0; HEADER_SIZE as usize];

        let (opcode, length) = buf.split_at_mut(size_of::<OpCode>());
        // yes, conversion to big endian is useless here, but if the opcode type changes sometime
        // later this will be needed
        opcode[0] = (self.opcode as u8).to_be();
        let length_bytes = self.length.to_be_bytes();
        length.copy_from_slice(&length_bytes);

        buf
    }

    /// Parse the header from a provided buffer
    pub fn deserialize(buf: &[u8]) -> crate::Result<Self> {
        if buf.len() < HEADER_SIZE.into() {
            return Err(crate::Error::UnknownMessage);
        }

        let (opcode_buf, length_buf) = buf.split_at(size_of::<OpCode>());
        // TODO: check if this behaves right on little endian
        let opcode = if cfg!(target_endian = "big") {
            opcode_buf[0]
        } else {
            opcode_buf[0].to_le()
        };
        let opcode = OpCode::from_repr(opcode).ok_or(crate::Error::UnknownMessage)?;

        // TODO: also check if this behaves right on little endian
        let mut length: u16 = 0;
        length += u16::from(length_buf[0] << 1);
        length += u16::from(length_buf[1]);

        Ok(Self { opcode, length })
    }
}

#[cfg(test)]
mod header_tests {
    use crate::message::{HEADER_SIZE, MessageHeader, OpCode};

    #[test]
    fn serialize_header() {
        const LENGTH: u16 = 69;

        let header = MessageHeader {
            opcode: OpCode::StartMessage,
            length: LENGTH,
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

        let mut header_buf = [0; HEADER_SIZE as usize];

        let (opcode_field, length_field) = header_buf.split_at_mut(size_of::<OpCode>());
        opcode_field[0] = OPCODE as u8;
        length_field.copy_from_slice(&LENGTH.to_be_bytes());

        let header = MessageHeader::deserialize(&header_buf)?;
        assert_eq!(header.opcode, OPCODE);
        assert_eq!(header.length, LENGTH);

        Ok(())
    }

    #[test]
    fn deserialize_header_invalid_opcode() -> crate::Result<()> {
        const OPCODE: u8 = u8::MAX;

        // length field is already set to 0
        let mut header_buf = [0; HEADER_SIZE as usize];
        header_buf[0] = OPCODE;

        if let Ok(msg) = MessageHeader::deserialize(&header_buf) {
            panic!("message shouldn't have been deserialized (deserialized to {msg:?})");
        }

        Ok(())
    }
}

/// A message to be sent or received over the network using [`crate::Connection`]
#[derive(Debug)]
pub enum Message {
    /// A message indicating a container should be started
    StartMessage {
        /// Name/ID of the container to be started
        resource_name: String,
    },
}

impl Message {
    /// Create a header from the current message
    pub fn create_header(&self) -> crate::Result<MessageHeader> {
        Ok(match self {
            Self::StartMessage { resource_name } => MessageHeader {
                opcode: OpCode::StartMessage,
                length: resource_name
                    .len()
                    .try_into()
                    .ok()
                    .ok_or(crate::Error::MesssageTooBig)?,
            },
        })
    }

    /// Serialize the payload data into bytes. This doesn't include the header; you have to
    /// construct the header separately
    pub fn serialize_payload(&self) -> crate::Result<Box<[u8]>> {
        match self {
            Self::StartMessage { resource_name } => {
                if resource_name.len() > MAX_MESSAGE_PAYLOAD_LENGTH.into() {
                    return Err(crate::Error::MesssageTooBig);
                }

                Ok(resource_name.as_bytes().to_vec().into_boxed_slice())
            }
        }
    }

    /// Deserialize a message from a buffer. `payload_buf.len()` is assumed to be `<= MAX_MESSAGE_LENGTH`
    pub fn deserialize(header: &MessageHeader, payload_buf: &[u8]) -> crate::Result<Self> {
        if payload_buf.len() > MAX_MESSAGE_PAYLOAD_LENGTH.into() {
            return Err(crate::Error::MesssageTooBig);
        }

        assert_eq!(payload_buf.len(), header.length.into());

        match header.opcode {
            OpCode::StartMessage => {
                let resource_name = &payload_buf[0..header.length.into()];
                let resource_name = str::from_utf8(resource_name)?.to_owned();

                Ok(Self::StartMessage { resource_name })
            }
        }
    }
}

#[cfg(test)]
mod message_tests {
    use crate::Message;

    #[test]
    fn serde_start_message() -> crate::Result<()> {
        let resource_name = "my_resource";

        let message = Message::StartMessage {
            resource_name: resource_name.to_owned(),
        };
        let header = message.create_header()?;
        let message = message.serialize_payload()?;

        let message = Message::deserialize(&header, &message)?;

        #[allow(irrefutable_let_patterns)] // TODO: remove this when more message types are added
        if let Message::StartMessage {
            resource_name: parsed_res_name,
        } = message
        {
            assert_eq!(resource_name, parsed_res_name.as_str());
        } else {
            panic!("Start message in buffer deserialized to a different type")
        }

        Ok(())
    }

    #[test]
    fn serialize_start_message_payload_too_big() -> crate::Result<()> {
        let fill_char = 'A';
        let s = fill_char.to_string().repeat((u16::MAX as usize) + 1);

        if let Ok(msg) = (Message::StartMessage { resource_name: s }).serialize_payload() {
            panic!("message serialized with overly long payload: {msg:?}")
        }

        Ok(())
    }
}
