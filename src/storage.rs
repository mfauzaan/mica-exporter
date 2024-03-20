use std::{fmt, sync::Arc};

pub mod s3;

/// Generic, shared error type.
///
/// As the underlying error type used by the implementation is not known, this error type is used
/// to allow errors to be cached when appropriate. Using an [`Arc`] here allows the error to be
/// cloned and stored, while retaining as much information as possible.
pub type SharedError = Arc<dyn std::error::Error + Send + Sync>;

/// Error putting a package into storage.
///
/// This classifies the errors produced downstream according to their semantics. The only error we
/// really care about at the moment is the `PackageMissing` case, because that one has different
/// caching semantics than other errors.
#[derive(thiserror::Error, Debug, Clone)]
pub enum StorageError {
    /// Package missing
    #[error("package missing")]
    PackageMissing(#[source] SharedError),

    /// Unknown error
    #[error(transparent)]
    Other(#[from] SharedError),
}

/// # Storage for package sources
///
/// This trait specifies a generic storage implementation for package sources
///
/// ## Error handling
///
pub trait Storage: Send + Sync + fmt::Debug {
    /// Write package to storage.
    ///
    /// In general, packages are immutable once stored. However, the semantics of this call are
    /// those of overwrite. Refer to the documentation of the trait for more context.
    async fn save_object(&self, key: &str, data: &[u8]) -> Result<(), StorageError>;

    /// Get package from storage.
    ///
    /// If the package does not exist, this will return a [`StorageError::PackageMissing`]. This
    /// call should only succeed once the package has been successfully written.
    async fn get_object(&self, key: &str) -> Result<Vec<u8>, StorageError>;
}
