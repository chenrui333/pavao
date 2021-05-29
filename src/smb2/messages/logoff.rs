//! # logoff
//!
//! this modules exposes the data types for the Logoff command

/**
 * MIT License
 *
 * pavao - Copyright (C) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
// locals
use super::{Command, CommandId, Decode, Encode, Error, SmbResult};
// deps
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// ## LogoffRequest
///
/// Represents a LogoffRequest
pub struct LogoffRequest {
    struct_size: u16,
    rfu: u16,
}

impl Default for LogoffRequest {
    fn default() -> Self {
        Self {
            struct_size: 0x04,
            rfu: 0x00,
        }
    }
}

impl Encode for LogoffRequest {
    fn encode(&self) -> Bytes {
        let mut buff: BytesMut = BytesMut::with_capacity(4);
        buff.put_u16(self.struct_size);
        buff.put_u16(self.rfu);
        buff.freeze()
    }
}

impl Command for LogoffRequest {
    fn command_id(&self) -> CommandId {
        CommandId::Logoff
    }
}

/// ## LogoffRequest
///
/// Represents a LogoffResponse
pub struct LogoffResponse;

impl Decode for LogoffResponse {
    fn decode(buff: &mut dyn Buf) -> SmbResult<Self> {
        if buff.remaining() < 4 {
            return Err(Error::InvalidSyntax);
        }
        buff.advance(4);
        Ok(LogoffResponse {})
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_smb2_messages_logoff_encode() {
        let req: LogoffRequest = LogoffRequest::default();
        let mut buff: Bytes = req.encode();
        assert_eq!(buff.remaining(), 0x04);
        assert_eq!(buff.get_u16(), 0x04);
        assert_eq!(buff.get_u16(), 0x00);
    }

    #[test]
    fn test_smb2_messages_logoff_decode() {
        let mut buff: Bytes = Bytes::from(vec![0x00, 0x04, 0x00, 0x00]);
        assert!(LogoffResponse::decode(&mut buff).is_ok());
        let mut buff: Bytes = Bytes::from(vec![0x00, 0x04]);
        assert!(LogoffResponse::decode(&mut buff).is_err());
    }
}