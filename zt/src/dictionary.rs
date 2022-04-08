use std::fmt;
use std::mem;
use arrayref::array_ref;
use failure::{Fallible, Fail};

const U64_SIZE: usize = mem::size_of::<u64>();
const I64_SIZE: usize = mem::size_of::<i64>();

#[derive(Debug, Fail, FromPrimitive)]
pub enum DictionaryError {
    #[fail(display = "not found")]
    NotFound,
    #[fail(display = "wrong type")]
    WrongType,
}

pub struct Dictionary<'a>(&'a [u8]);

impl<'a> Dictionary<'a> {
    pub fn from(buf: &'a [u8]) -> Self {
        Self(buf)
    }

    pub fn get_u64(&self, key: &str) -> Fallible<u64> {
        let val = self.get_key(key)?;
        let dec = hex::decode(val)?;
        if dec.len() == U64_SIZE {
            return Ok(u64::from_be_bytes(*array_ref!(dec, 0, U64_SIZE)));
        }
        Err(DictionaryError::WrongType.into())
    }

    pub fn get_i64(&self, key: &str) -> Fallible<i64> {
        let val = self.get_key(key)?;
        let dec = hex::decode(val)?;
        if dec.len() == I64_SIZE {
            return Ok(i64::from_be_bytes(*array_ref!(dec, 0, I64_SIZE)));
        }
        Err(DictionaryError::WrongType.into())
    }

    fn get_key(&self, key: &str) -> Fallible<&[u8]> {
        let iter = self.0.split(|item| *item == '\n' as u8);
        for pair in iter {
            if pair.len() > 0 && pair[0] == 0 {
                break;
            }
            if let Some(i) = pair.iter().position(|x| *x == '=' as u8) {
                if &pair[..i] == key.as_bytes() {
                    return Ok(&pair[i+1..]);
                }
            }
        };
        Err(DictionaryError::NotFound.into())
    }
}

impl<'a> fmt::Debug for Dictionary<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let iter = self.0.split(|item| *item == '\n' as u8);
        for pair in iter {
            if pair.len() > 0 && pair[0] != 0 {
                if let Ok(p) = std::str::from_utf8(pair) {
                    if p.trim().len() > 0 { write!(f, "{}\n", p)?; }
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}
