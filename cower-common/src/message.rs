//! Code related to messages that clients and servers can pass to one another.

/// The length of the `opcode` field in serialized messages
pub const OPCODE_LENGTH: usize = size_of::<u8>();
/// The length of the `length` field in serialized messages
pub const LENGTH_FIELD_LENGTH: usize = size_of::<u8>();
/// The total message length
pub const MESSAGE_LENGTH: usize = u8::MAX as usize;

/// A message that can serialize itself into a byte buffer as well as deserialize itself from one
pub trait Message {
    /// Serialize into bytes ready to be send over the network
    fn serialize(&self) -> crate::Result<[u8; MESSAGE_LENGTH]>;
    /// Deserialize from a byte buffer
    fn deserialize(data: &[u8; MESSAGE_LENGTH]) -> crate::Result<Box<Self>>
    where
        Self: Sized;
}

#[derive(strum::FromRepr)]
#[repr(u8)]
enum OpCode {
    StartMessage = 0,
}

/// A message that tells the receiver that it should start a container
#[derive(Debug, PartialEq, Eq)]
pub struct StartMessage {
    /// The name of the resource/container to be started
    pub resource_name: String,
}

impl Message for StartMessage {
    fn serialize(&self) -> crate::Result<[u8; MESSAGE_LENGTH]> {
        if self.resource_name.len() > MESSAGE_LENGTH - OPCODE_LENGTH - LENGTH_FIELD_LENGTH {
            return Err(crate::Error::MesssageTooBig);
        }

        let mut buf = [0; MESSAGE_LENGTH as usize];
        let (opcode_field, data_section) = buf.split_at_mut(OPCODE_LENGTH);
        assert!(opcode_field.len() == OPCODE_LENGTH);

        let (length_field, data_field) = data_section.split_at_mut(LENGTH_FIELD_LENGTH);
        assert!(length_field.len() == LENGTH_FIELD_LENGTH);

        opcode_field[0] = OpCode::StartMessage as u8;
        let name = self.resource_name.as_bytes();
        length_field[0] = self
            .resource_name
            .len()
            .try_into()
            .expect("Resource name length is too long");
        data_field[0..self.resource_name.len()].copy_from_slice(name);

        return Ok(buf);
    }

    fn deserialize(data: &[u8; MESSAGE_LENGTH]) -> crate::Result<Box<Self>>
    where
        Self: Sized,
    {
        let (opcode, data_section) = data.split_at(OPCODE_LENGTH);
        assert!(opcode.len() == OPCODE_LENGTH);

        if opcode[0] != OpCode::StartMessage as u8 {
            return Err(crate::Error::UnknownMessage);
        }

        let (length_field, data_field) = data_section.split_at(1);
        assert!(length_field.len() == LENGTH_FIELD_LENGTH);
        let length = length_field[0];

        let resource_name_section = &data_field[0..(length as usize)];
        let resource_name = str::from_utf8(resource_name_section)?.to_owned();

        Ok(Self {
            resource_name: resource_name,
        }
        .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_start_message() {
        let name = "my_resource";
        let msg = StartMessage {
            resource_name: name.to_owned(),
        };

        let ser = msg.serialize().expect("Failed to serialize message");

        let de = StartMessage::deserialize(ser.as_ref().try_into().unwrap()).unwrap();

        assert_eq!(msg, *de);
    }
}
