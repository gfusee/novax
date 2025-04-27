use crate::{DecodeError, TopDecodeMulti};

pub trait DecodableEvent: Sized {
    fn decode_event(topics: Vec<Vec<u8>>, data: Vec<u8>) -> Result<Self, DecodeError>;
}

impl<T: TopDecodeMulti> DecodableEvent for T {
    fn decode_event(_topics: Vec<Vec<u8>>, data: Vec<u8>) -> Result<Self, DecodeError> {
        Self::multi_decode(&mut vec![data])
    }
}