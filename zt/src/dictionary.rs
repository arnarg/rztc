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

#[derive(Clone)]
pub struct Dictionary(Vec<u8>);

impl Dictionary {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from(buf: Vec<u8>) -> Self {
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

    pub fn set_u64(&mut self, key: &str, value: u64) {
        self.0.append(&mut format!("{}={:016x}\n", key, value).into_bytes());
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.0.append(&mut format!("{}={}\n", key, if value { 1 } else { 0 }).into_bytes());
    }

    pub fn set_str(&mut self, key: &str, value: &str) {
        self.set_bytes(key, value.as_bytes());
    }

    pub fn set_bytes(&mut self, key: &str, value: &[u8]) {
        self.0.append(&mut format!("{}=", key).into_bytes());
        for &c in value {
            match c {
                0 =>  self.0.append(&mut "\\0".as_bytes().to_vec()),  // Binary 0
                13 => self.0.append(&mut "\\r".as_bytes().to_vec()),  // \r
                10 => self.0.append(&mut "\\n".as_bytes().to_vec()),  // \n
                92 => self.0.append(&mut "\\\\".as_bytes().to_vec()), // \
                61 => self.0.append(&mut "\\e".as_bytes().to_vec()),  // =
                _ =>  self.0.push(c),
            }
        }
        self.0.push(10); // \n
    }

    pub fn finalize(&self) -> Vec<u8> {
        let mut buf = self.0.clone();
        buf.push(0);
        buf
    }
}

impl fmt::Debug for Dictionary {
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
