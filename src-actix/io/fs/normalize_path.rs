use std::path::PathBuf;

/// The `NormalizePath` trait defines a method for converting a type into an
/// operating-system-compatible path representation (`std::path::PathBuf`).
///
/// This trait can be implemented for different types to streamline their
/// conversion to a `PathBuf` in a consistent and platform-compatible manner.
pub trait NormalizePath {
    /// Converts the current object into a `PathBuf` that is compatible with the operating system.
    ///
    /// This method is typically used to translate an internal representation of a path or an
    /// abstract path type into a concrete `PathBuf`, which is the standard type for handling filesystem
    /// paths in Rust. The resulting path can then be used with Rust's filesystem manipulation APIs
    /// or other OS-specific functionalities.
    ///
    /// # Returns
    ///
    /// A `PathBuf` instance representing the path in a format that is valid for the operating system.
    ///
    /// # Examples
    ///
    /// ```norust
    /// use std::path::PathBuf;
    ///
    /// // Assuming an implementation of `to_os_path` for a custom type
    /// let custom_path = SomeCustomPathType::new("/some/path");
    /// let os_path: PathBuf = custom_path.to_os_path();
    ///
    /// assert_eq!(os_path, PathBuf::from("/some/path"));
    /// ```
    ///
    /// # Notes
    ///
    /// - The specific conversion logic depends on the implementation for the type that
    ///   this method is defined on.
    /// - Behaviors such as normalization, validation, or error handling should be
    ///   documented in the concrete implementation of this trait/method.
    fn to_os_path(&self) -> PathBuf;
}

impl NormalizePath for String {
    fn to_os_path(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            PathBuf::from(if let Some(stripped) = self.strip_prefix("/") {
                stripped
            } else {
                self
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            if !self.starts_with("/") {
                PathBuf::from(format!("/{}", self))
            } else {
                PathBuf::from(self)
            }
        }
    }
}
impl NormalizePath for PathBuf {
    fn to_os_path(&self) -> PathBuf {
        self.to_string_lossy().to_string().to_os_path()
    }
}

impl NormalizePath for &str {
    fn to_os_path(&self) -> PathBuf {
        self.to_string().to_os_path()
    }
}
