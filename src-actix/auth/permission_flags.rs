use enumflags2::bitflags;
use serde::{Deserialize, Serialize};

#[bitflags]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum PermissionFlags {
    Read = 0b00000001,
    Write = 0b00000010,
    Delete = 0b00000100,
    Create = 0b00001000,
    Upload = 0b00010000,
    Download = 0b00100000,
}

impl Default for PermissionFlags {
    fn default() -> Self {
        Self::Read
    }
}

impl PermissionFlags {
    pub fn all() -> u8 {
        (Self::Read | Self::Write | Self::Delete | Self::Create | Self::Upload | Self::Download)
            .bits_c()
    }
}
