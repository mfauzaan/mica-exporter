use crate::storage::{Storage, StorageError};
use aws_sdk_s3::{
    error::SdkError,
    operation::get_object::GetObjectError,
    primitives::{ByteStream, SdkBody},
    Client,
};
use std::sync::Arc;

/// # S3-backed package storage.
///
/// This storage implementation keeps packages in an S3 bucket using the `aws_sdk` crate. The
/// packages are named similar to how they are named in the filesystem.
///
/// For example, a package named `mypackage` with version `0.1.5` would be stored as
/// `mypackage_0.1.5.tar.gz` in the bucket.
#[derive(Clone, Debug)]
pub struct S3 {
    client: Client,
    bucket: String,
}

impl S3 {
    /// Create new instance given an S3 [`Client`] and a bucket name.
    pub fn new(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

impl Storage for S3 {
    async fn save_object(&self, key: &str, data: &[u8]) -> Result<(), StorageError> {
        println!("Saving object to S3: {}", key);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::new(SdkBody::from(data)))
            .send()
            .await
            .map(|_| ())
            .map_err(|error| StorageError::Other(Arc::new(error)))
    }

    async fn get_object(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await;

        // determine if this is a no such key error and translate into package missing
        if let Err(SdkError::ServiceError(error)) = &response {
            if let GetObjectError::NoSuchKey(error) = error.err() {
                return Err(StorageError::PackageMissing(Arc::new(error.clone())));
            }
        }

        // return other errors as-is
        let response = match response {
            Ok(response) => response,
            Err(error) => return Err(StorageError::Other(Arc::new(error))),
        };

        // collect response
        response
            .body
            .collect()
            .await
            .map_err(|error| StorageError::Other(Arc::new(error)))
            .map(|data| data.to_vec())
    }
}
