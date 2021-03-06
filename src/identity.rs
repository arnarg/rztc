use std::cell::Cell;
use zt::core::{StateProvider, StateObject, StateError};
use failure::Fallible;

const IDENTITY_LENGTH: usize = 270;

pub struct IdentityState {
    identity_file: Box<String>,
    identity: Cell<[u8; IDENTITY_LENGTH]>,
}

impl IdentityState {
    pub fn new(identity_file: &str) -> Self {
        Self {
            identity_file: Box::new(identity_file.to_string()),
            identity: Cell::new([0u8; IDENTITY_LENGTH]),
        }
    }

    fn get_identity(&self) -> Fallible<[u8; IDENTITY_LENGTH]> {
        if self.identity.get()[..5] != [0,0,0,0,0] {
            return Ok(self.identity.get());
        }
        // Reading identity from identity_file
        let id = std::fs::read_to_string(self.identity_file.as_str())?;
        // Copying the contents of the file to a buffer and setting
        // the contents as the identity
        let mut buf = [0u8; IDENTITY_LENGTH];
        buf.copy_from_slice(&id.trim().as_bytes()[..IDENTITY_LENGTH]);
        self.identity.set(buf);
        Ok(self.identity.get())
    }

    fn set_identity(&self, buf: &[u8]) -> Fallible<()> {
        if buf.len() <= IDENTITY_LENGTH {
            std::fs::write(self.identity_file.as_str(), buf)?;
            self.identity.get().copy_from_slice(buf);
            return Ok(());
        }
        Err(StateError::TooLong.into())
    }
}

impl StateProvider for IdentityState {
    fn get_state(&self, object_type: StateObject) -> Fallible<Vec<u8>> {
        let res = match object_type {
            StateObject::PublicIdentity => Vec::from(&self.get_identity()?[..141]),
            StateObject::SecretIdentity => Vec::from(&self.get_identity()?[..]),
        };
        Ok(res)
    }

    fn set_state(&self, object_type: StateObject, data: &[u8]) -> Fallible<()> {
        let _res = match object_type {
            StateObject::SecretIdentity => self.set_identity(data),
            _ => Err(StateError::NotFound.into()),
        };
        Ok(())
    }
}
