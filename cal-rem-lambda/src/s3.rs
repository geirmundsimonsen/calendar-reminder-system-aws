use futures::stream::TryStreamExt;
use lambda_runtime::Error;
use s3::{ByteStream, Client};

pub async fn get_object_as_string(bucket: String, key: String) -> Result<String, Error> {
    let client = Client::from_env();
    let res = client.get_object().bucket(bucket).key(key).send().await?;
    let body = res.body.map_ok(|b| b.to_vec()).try_concat().await?;
    Ok(String::from_utf8(body)?)
}

pub async fn save_string_as_object(s: String, bucket: String, key: String) -> Result<(), Error> {
    let client = Client::from_env();
    let buffer = ByteStream::from(Vec::from(s.as_bytes()));
    let _f = client.put_object().bucket(bucket).key(key).body(buffer).send().await?;

    Ok(())
}