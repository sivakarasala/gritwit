use crate::configuration::StorageSettings;
use secrecy::ExposeSecret;

pub enum StorageBackend {
    Local,
    R2 {
        bucket: Box<s3::Bucket>,
        public_url: String,
    },
}

impl StorageBackend {
    pub fn from_config(config: &StorageSettings) -> Self {
        match config.backend.as_str() {
            "r2" => {
                let account_id = config
                    .r2_account_id
                    .as_ref()
                    .expect("R2 account_id required");
                let access_key = config
                    .r2_access_key
                    .as_ref()
                    .expect("R2 access_key required")
                    .expose_secret()
                    .clone();
                let secret_key = config
                    .r2_secret_key
                    .as_ref()
                    .expect("R2 secret_key required")
                    .expose_secret()
                    .clone();
                let bucket_name = config
                    .r2_bucket
                    .as_ref()
                    .expect("R2 bucket required");
                let public_url = config
                    .r2_public_url
                    .as_ref()
                    .expect("R2 public_url required")
                    .clone();

                let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);
                let region = s3::Region::Custom {
                    region: "auto".to_string(),
                    endpoint,
                };
                let credentials = s3::creds::Credentials::new(
                    Some(&access_key),
                    Some(&secret_key),
                    None,
                    None,
                    None,
                )
                .expect("Failed to create R2 credentials");

                let bucket = s3::Bucket::new(bucket_name, region, credentials)
                    .expect("Failed to create R2 bucket")
                    .with_path_style();

                StorageBackend::R2 { bucket, public_url }
            }
            _ => StorageBackend::Local,
        }
    }

    pub async fn upload(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<String, String> {
        match self {
            StorageBackend::Local => {
                let upload_dir = std::path::Path::new("public/videos");
                tokio::fs::create_dir_all(upload_dir)
                    .await
                    .map_err(|e| format!("Failed to create upload dir: {}", e))?;
                let filepath = upload_dir.join(key);
                tokio::fs::write(&filepath, data)
                    .await
                    .map_err(|e| format!("Failed to save file: {}", e))?;
                Ok(format!("/videos/{}", key))
            }
            StorageBackend::R2 { bucket, public_url } => {
                let path = format!("videos/{}", key);
                bucket
                    .put_object_with_content_type(&path, data, content_type)
                    .await
                    .map_err(|e| format!("R2 upload failed: {}", e))?;
                Ok(format!("{}/{}", public_url.trim_end_matches('/'), path))
            }
        }
    }
}
