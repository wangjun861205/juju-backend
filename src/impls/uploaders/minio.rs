use aws_config;
use aws_sdk_s3::Client;

use crate::core::uploader::Uploader;
pub struct Minio {
    bucket: String,
    client: Client,
}

impl Uploader for Minio {
    type ID = String;
    async fn bulk_delete(&mut self, ids: Vec<Self::ID>) -> Result<(), crate::error::Error> {
        for id in ids {
            self.client.delete_object().bucket(&self.bucket).key(&id).send().await?;
        }
        Ok(())
    }
}
