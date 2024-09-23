use tokio_util::{
    bytes::{Buf, BufMut, BytesMut},
    codec::{Decoder, Encoder},
};

use crate::frame::{Action, TelnetFrame, TelnetOption, TelnetSubnegotiation};

pub struct TelnetCodec {}

impl TelnetCodec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetFrame;
    type Error = tokio::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let byte = src[0];

        let frame = match byte {
            255 => {
                if src.len() < 2 {
                    return Ok(None);
                }

                match Action::try_from(src[1]) {
                    Ok(Action::SubnegotiationBegin) => {
                        // Find the end of the subnegotiation by a sequence of IAC SE
                        match src.windows(2).position(|window| window == [255, 240]) {
                            Some(end) => {
                                // Skip the IAC SB
                                src.advance(2);

                                let data = src.split_to(end - 2).to_vec();

                                src.advance(2);

                                TelnetFrame::Subnegotiation(TelnetSubnegotiation::from(data))
                            }

                            None => return Ok(None),
                        }
                    }

                    Ok(action) => {
                        if src.len() < 3 {
                            return Ok(None);
                        }

                        let option = TelnetOption::from(src[2]);
                        src.advance(3);

                        TelnetFrame::Command { action, option }
                    }

                    Err(_) => {
                        if src[1] == 255 {
                            src.advance(1);
                            TelnetFrame::Data(vec![255])
                        } else {
                            let data = src.split_to(2).to_vec();
                            TelnetFrame::Data(data)
                        }
                    }
                }
            }

            _ => {
                let end = src.iter().position(|&b| b == 255).unwrap_or(src.len());
                let data = src.split_to(end).to_vec();

                TelnetFrame::Data(data)
            }
        };

        Ok(Some(frame))
    }
}

impl Encoder<TelnetFrame> for TelnetCodec {
    type Error = tokio::io::Error;

    fn encode(&mut self, item: TelnetFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            TelnetFrame::Data(data) => dst.extend(data),

            TelnetFrame::Command { action, option } => {
                dst.put_u8(255);
                dst.put_u8(action.into());
                dst.put_u8(option.into());
            }

            TelnetFrame::Subnegotiation(subnegotiation) => {
                dst.put_u8(255);
                dst.put_u8(250);
                dst.extend::<Vec<u8>>(subnegotiation.into());
                dst.put_u8(255);
                dst.put_u8(240);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::TelnetOption;

    use super::*;

    #[test]
    fn test_decode_data_frame() {
        let mut codec = TelnetCodec::new();
        let mut buf = BytesMut::from(&b"hello"[..]);

        let frame = codec.decode(&mut buf).unwrap().unwrap();
        match frame {
            TelnetFrame::Data(data) => assert_eq!(data, b"hello"),
            _ => panic!("Expected TelnetFrame::Data"),
        }
    }

    #[test]
    fn test_decode_command_frame() {
        let mut codec = TelnetCodec::new();
        let mut buf = BytesMut::from(&[255, 251, 1][..]);

        let frame = codec.decode(&mut buf).unwrap().unwrap();
        match frame {
            TelnetFrame::Command { action, option } => {
                assert_eq!(action, Action::Will);
                assert_eq!(option, TelnetOption::SuppressGoAhead);
            }
            _ => panic!("Expected TelnetFrame::Command"),
        }
    }

    #[test]
    fn test_decode_escaped_255() {
        let mut codec = TelnetCodec::new();
        let mut buf = BytesMut::from(&[255, 255][..]);

        let frame = codec.decode(&mut buf).unwrap().unwrap();
        match frame {
            TelnetFrame::Data(data) => assert_eq!(data, vec![255]),
            _ => panic!("Expected TelnetFrame::Data"),
        }
    }

    #[test]
    fn test_decode_partial_command() {
        let mut codec = TelnetCodec::new();
        let mut buf = BytesMut::from(&[255, 251][..]);

        let frame = codec.decode(&mut buf).unwrap();
        assert!(frame.is_none());
    }

    #[test]
    fn test_decode_data_followed_by_command() {
        let mut codec = TelnetCodec::new();
        let mut buf = BytesMut::from(&[104, 101, 108, 108, 111, 255, 251, 1][..]);

        let frame = codec.decode(&mut buf).unwrap().unwrap();
        match frame {
            TelnetFrame::Data(data) => assert_eq!(data, b"hello"),
            _ => panic!("Expected TelnetFrame::Data"),
        }

        let frame = codec.decode(&mut buf).unwrap().unwrap();
        match frame {
            TelnetFrame::Command { action, option } => {
                assert_eq!(action, Action::Will);
                assert_eq!(option, TelnetOption::Echo);
            }
            _ => panic!("Expected TelnetFrame::Command"),
        }
    }
}
