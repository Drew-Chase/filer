use anyhow::Error;
use enumflags2::{BitFlags, bitflags};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};

/// Represents a set of permission flags using a bitmask.
///
/// `PermissionFlags` is an enum that defines various permissions typically used
/// for file or resource access control. Each permission is represented as a distinct
/// bit in an 8-bit unsigned integer (u8), allowing combinations of permissions to be
/// stored compactly. This enum can be used with bitwise operations to check or manipulate
/// permissions efficiently.
///
/// # Variants
///
/// * `Read` (0b00000001) - Permission to read the resource.
/// * `Write` (0b00000010) - Permission to write or modify the resource.
/// * `Delete` (0b00000100) - Permission to delete the resource.
/// * `Create` (0b00001000) - Permission to create a new resource.
/// * `Upload` (0b00010000) - Permission to upload files or data.
/// * `Download` (0b00100000) - Permission to download files or data.
///
/// # Attributes
///
/// * `#[bitflags]` - Enables usage of the enum as a bitmask, providing utility methods
///   for working with combinations of permission flags.
/// * `#[repr(u8)]` - Specifies that the enum is represented as an 8-bit unsigned integer.
/// * `#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]` - Implements
///   traits for debugging, copying, cloning, and serializing/deserializing the enum.
///
/// # Example
///
/// ```
/// use my_crate::PermissionFlags;
///
/// // Define a set of permissions using bitwise OR.
/// let permissions = PermissionFlags::Read as u8 | PermissionFlags::Write as u8;
///
/// // Check if a permission includes the `Read` flag.
/// if permissions & PermissionFlags::Read as u8 > 0 {
///     println!("Read permission is granted.");
/// }
///
/// // Check if a permission includes the `Delete` flag.
/// if permissions & PermissionFlags::Delete as u8 == 0 {
///     println!("Delete permission is not granted.");
/// }
/// ```
///
/// This enum is ideal for scenarios where compact representation of multiple
/// binary state flags is required, such as in file system operations or access control systems.
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
    /// Implements `Default` trait for `PermissionFlags`.
    ///
    /// Returns `PermissionFlags::Read` as the default permission,
    /// providing minimal read-only access.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::auth::PermissionFlags;
    ///
    /// let default_permissions = PermissionFlags::default();
    /// assert_eq!(default_permissions, PermissionFlags::Read);
    /// ```
    fn default() -> Self {
        Self::Read
    }
}

impl PermissionFlags {
    /// Returns a `u8` value representing all the available permission flags combined.
    ///
    /// This function combines all predefined permission flags (`Read`, `Write`, `Delete`,
    /// `Create`, `Upload`, and `Download`) into a single bitmask representation
    /// and returns the corresponding `u8` value using the `bits_c` method.
    ///
    /// # Returns
    ///
    /// * `u8`: A numerical representation where all permission flags are set.
    ///
    /// # Example
    ///
    /// ```rust
    /// let all_permissions = Permissions::all();
    /// println!("All permissions: {:#010b}", all_permissions);
    /// // Output might be: 0b00111111 (if 6 permissions combined starting from least significant bit)
    /// ```
    pub fn all() -> u8 {
        (Self::Read | Self::Write | Self::Delete | Self::Create | Self::Upload | Self::Download).bits_c()
    }
    /// Converts permissions strings into a `BitFlags<PermissionFlags>`.
    ///
    /// # Parameters
    ///
    /// - `permissions`: A slice of `String` where each string represents a permission.
    /// The valid permission strings are:
    ///   - `"Read"`
    ///   - `"Write"`
    ///   - `"Delete"`
    ///   - `"Create"`
    ///   - `"Upload"`
    ///   - `"Download"`
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `BitFlags<PermissionFlags>`:
    /// A bitflags object representing the accumulated permissions on success.
    /// - `anyhow::Error`: An error if an invalid permission string is encountered.
    ///
    /// # Errors
    ///
    /// An error is returned
    /// if the input slice contains a string
    /// that does not match any of the valid permission strings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::auth::{PermissionFlags, from_strings};
    /// use bitflags::BitFlags;
    ///
    /// let permissions = vec![
    ///     String::from("Read"),
    ///     String::from("Write"),
    ///     String::from("Delete"),
    /// ];
    ///
    /// let flags = from_strings(&permissions).unwrap();
    /// assert!(flags.contains(PermissionFlags::Read));
    /// assert!(flags.contains(PermissionFlags::Write));
    /// assert!(flags.contains(PermissionFlags::Delete));
    /// ```
    pub fn from_strings(permissions: &[String]) -> anyhow::Result<BitFlags<PermissionFlags>> {
        let mut flags = BitFlags::empty();
        debug!("Processing permission flags from input: {:?}", permissions);
        for permission in permissions {
            match permission.as_str() {
                "Read" => {
                    flags |= PermissionFlags::Read;
                    trace!("Added Read permission flag");
                }
                "Write" => {
                    flags |= PermissionFlags::Write;
                    trace!("Added Write permission flag");
                }
                "Delete" => {
                    flags |= PermissionFlags::Delete;
                    trace!("Added Delete permission flag");
                }
                "Create" => {
                    flags |= PermissionFlags::Create;
                    trace!("Added Create permission flag");
                }
                "Upload" => {
                    flags |= PermissionFlags::Upload;
                    trace!("Added Upload permission flag");
                }
                "Download" => {
                    flags |= PermissionFlags::Download;
                    trace!("Added Download permission flag");
                }
                _ => {
                    error!("Invalid permission encountered: {}", permission);
                    Err(Error::msg("Invalid permission"))?
                }
            }
        }
        info!("Permissions successfully processed, resulting flags: {:?}", flags);
        Ok(flags)
    }
}
