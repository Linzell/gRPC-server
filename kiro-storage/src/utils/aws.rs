// utils/aws.rs
//
// Copyright Charlie Cohen <linzellart@gmail.com>
//
// Licensed under the GNU General Public License, Version 3.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.gnu.org/licenses/gpl-3.0.html
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;

use aws_sdk_s3::{primitives::ByteStream, Client};

use clap::Parser;
use futures::future::join_all;
use kiro_database::{get_env_or, DbId};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::StorageError;

#[cfg(any(test, feature = "mock"))]
use mockall::{automock, predicate::*};

/// # Opt struct
///
/// The Opt struct is a struct that represents the options for the bucket.
#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The name of the bucket.
    #[structopt(short, long)]
    bucket: String,

    /// The object key.
    #[structopt(short, long)]
    object: String,

    /// How long in seconds before the presigned request should expire.
    #[structopt(short, long)]
    expires_in: Option<u64>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// # Bucket
///
/// The bucket module is a module that provides utilities for buckets.
pub struct BucketS3 {
    pub client: Client,
    pub region: String,
    pub bucket: String,
}

#[cfg_attr(any(test, feature = "mock"), automock)]
#[async_trait::async_trait]
pub trait BucketS3Trait: Send + Sync {
    async fn put_object(
        &self, object: ByteStream, source_id: DbId, request_path: &str, name: &str,
    ) -> Result<String, StorageError>;
}

#[async_trait::async_trait]
impl BucketS3Trait for BucketS3 {
    async fn put_object(
        &self, object: ByteStream, source_id: DbId, request_path: &str, name: &str,
    ) -> Result<String, StorageError> {
        self.put_object(object, source_id, request_path, name).await
    }
}

impl BucketS3 {
    /// # New
    ///
    /// The `new` method creates a new S3 bucket Client.
    ///
    /// ```rust,ignore
    /// let bucketS3 = BucketS3::new().await;
    ///
    /// println!("🪣 Bucket S3: {:?}", bucketS3);
    /// ```
    pub async fn new() -> Self {
        let bucket = get_env_or("AWS_S3_BUCKET_NAME", "your-bucket");
        let region = get_env_or("AWS_REGION", "eu-west-3");
        let aws_configuration = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region("eu-west-3") // Pass a reference to the region variable
            .load()
            .await;

        let client = Client::new(&aws_configuration);

        Self {
            client,
            region,
            bucket,
        }
    }

    /// # Get object
    ///
    /// The `get_object` method gets an object from the S3 bucket.
    ///
    /// ```rust,ignore
    /// let object = BucketS3::new().await.get_object(source_id, request_path, name).await?;
    ///
    /// println!("📦 Object: {:?}", object);
    /// ```
    pub async fn get_object(
        &self, source_id: DbId, request_path: &str, name: &str,
    ) -> Result<ByteStream, StorageError> {
        let key = format!("{}/{}_{}", &source_id.id, &request_path, &name);

        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(val) => Ok(val.body),
            Err(e) => Err(StorageError::S3GetError(e)),
        }
    }

    /// # Put object
    ///
    /// The `put_object` method puts an object in the S3 bucket.
    ///
    /// ```rust,ignore
    /// let object = BucketS3::new().await.put_object(object, source_id, request_path, name).await?;
    ///
    /// println!("📦 Object: {:?}", object);
    /// ```
    pub async fn put_object(
        &self, object: ByteStream, source_id: DbId, request_path: &str, name: &str,
    ) -> Result<String, StorageError> {
        let key = format!("{}/{}_{}", &source_id.id, &request_path, &name);

        match self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(object)
            .send()
            .await
        {
            Ok(_) => Ok(format!(
                "https://{}.s3.{}.amazonaws.com/{}/{}_{}",
                self.bucket, self.region, &source_id.id, &request_path, &name
            )),
            Err(e) => Err(StorageError::S3PutError(e)),
        }
    }

    /// # Put signed object
    ///
    /// The `put_signed_object` method puts a signed object in the S3 bucket.
    ///
    /// ```rust,ignore
    /// let url = BucketS3::new().await.put_signed_object(object, source_id, request_path, name, expires_in).await?;
    ///
    /// println!("🔏 Signed URL: {:?}", url);
    /// ```
    pub async fn put_signed_object(
        &self, object: ByteStream, source_id: DbId, request_path: &str, name: &str, expires_in: u64,
    ) -> Result<String, StorageError> {
        let key = format!("{}/{}_{}", &source_id.id, &request_path, &name);
        let presigned_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(
            std::time::Duration::from_secs(expires_in),
        )
        .expect("expires_in duration should be valid");

        match self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(object)
            .send()
            .await
        {
            Ok(_) => {
                // Create presigned URL for get request
                let url = self
                    .client
                    .get_object()
                    .bucket(&self.bucket)
                    .key(&key)
                    .presigned(presigned_config)
                    .await
                    .map_err(StorageError::S3GetError)?
                    .uri()
                    .to_string();

                Ok(url)
            }
            Err(e) => Err(StorageError::S3PutError(e)),
        }
    }

    /// # Prepare upload
    ///
    /// The `prepare_upload` method prepares an upload to the S3 bucket.
    ///
    /// ```rust,ignore
    /// let file = File::open("test.txt").await?;
    ///
    /// BucketS3::new().await.prepare_upload("test.txt".to_string(), file, source_id, request_path).await?;
    ///
    /// println!("📤 Prepared upload");
    /// ```
    async fn prepare_upload(
        &self, file_name: String, mut file: File, source_id: DbId, request_path: Arc<String>,
    ) -> Result<(), StorageError> {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(StorageError::IO)?;

        let object = ByteStream::from(buffer);

        self.put_object(object, source_id.clone(), &request_path, &file_name)
            .await?;
        Ok(())
    }

    /// # Put multiple objects
    ///
    /// The `put_multiple_objects` method puts multiple objects in the S3 bucket.
    /// Returns an URL for the folder.
    ///
    /// ```rust,ignore
    /// let files = vec![
    ///  ("test.txt".to_string(), File::open("test.txt").await?),
    /// ("test2.txt".to_string(), File::open("test2.txt").await?),
    /// ];
    ///
    /// let url = BucketS3::new().await.put_multiple_objects(files, source_id, request_path).await?;
    ///
    /// println!("🔗 URL: {:?}", url);
    /// ```
    pub async fn put_multiple_objects(
        &self, files: Vec<(String, File)>, source_id: DbId, request_path: &str,
    ) -> Result<String, StorageError> {
        let request_path = Arc::new(request_path.to_string());

        let upload_tasks: Vec<_> = files
            .into_iter()
            .map(|(file_name, file)| {
                let source_id = source_id.clone();
                let request_path = Arc::clone(&request_path);
                self.prepare_upload(file_name, file, source_id, request_path)
            })
            .collect();

        let results = join_all(upload_tasks).await;

        // Check if all uploads were successful
        for result in results {
            result?;
        }

        // Return the folder URL
        Ok(format!(
            "https://{}.s3.{}.amazonaws.com/{}/{}",
            &self.bucket, &self.region, &source_id.id, &request_path
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::primitives::ByteStream;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_bucket_s3_new() {
        let bucket = BucketS3::new().await;

        assert_eq!(bucket.bucket, "your-bucket");
        assert_eq!(bucket.region, "eu-west-3");
    }

    #[tokio::test]
    async fn test_bucket_s3_get_object() {
        let bucket = BucketS3::new().await;
        let source_id = DbId::from(("users", "test"));
        let request_path = "test";
        let name = "test";

        let object = bucket.get_object(source_id, request_path, name).await;

        assert!(object.is_err());
    }

    #[tokio::test]
    async fn test_bucket_s3_put_object() {
        let bucket = BucketS3::new().await;
        let source_id = DbId::from(("users", "test"));
        let request_path = "test";
        let name = "test";
        let object = ByteStream::from("test".as_bytes().to_vec());

        let url = bucket
            .put_object(object, source_id, request_path, name)
            .await;

        assert!(url.is_err());
    }

    #[tokio::test]
    async fn test_bucket_s3_put_presigned_object() {
        let bucket = BucketS3::new().await;
        let source_id = DbId::from(("users", "test"));
        let request_path = "test";
        let name = "test";
        let object = ByteStream::from("test".as_bytes().to_vec());
        let expires_in = 60;

        let url = bucket
            .put_signed_object(object, source_id, request_path, name, expires_in)
            .await;

        assert!(url.is_err());
    }

    #[tokio::test]
    async fn test_bucket_s3_put_multiple_objects() {
        let bucket = BucketS3::new().await;
        let source_id = DbId::from(("users", "test"));
        let request_path = "test";

        let mut file = File::create("test.txt").await.unwrap();
        file.write_all(b"test").await.unwrap();

        let mut file2 = File::create("test2.txt").await.unwrap();
        file2.write_all(b"test").await.unwrap();

        let files = vec![
            (
                "test.txt".to_string(),
                File::open("test.txt").await.unwrap(),
            ),
            (
                "test2.txt".to_string(),
                File::open("test2.txt").await.unwrap(),
            ),
        ];

        std::fs::remove_file("test.txt").unwrap();
        std::fs::remove_file("test2.txt").unwrap();

        let url = bucket
            .put_multiple_objects(files, source_id, request_path)
            .await;

        assert!(url.is_err());
    }
}
