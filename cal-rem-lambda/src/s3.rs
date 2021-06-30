use lambda_runtime::Error;
use rusoto_s3::{GetObjectRequest, PutObjectRequest, S3, StreamingBody};
use futures::stream::TryStreamExt;

pub async fn get_object_as_string(bucket: String, key: String) -> Result<String, Error> {
    let c = rusoto_s3::S3Client::new(rusoto_core::Region::EuNorth1);
    let f = c.get_object(GetObjectRequest { bucket, key, ..Default::default() });
    let mut obj = f.await?;
    let body = obj.body.take().expect("The object has no body");
    let body = body.map_ok(|b| b.to_vec()).try_concat().await?;
    
    let str = String::from_utf8(body)?;

    Ok(str)
}

pub async fn save_string_as_object(s: String, bucket: String, key: String) -> Result<(), Error> {
    let c = rusoto_s3::S3Client::new(rusoto_core::Region::EuNorth1);
    
    let buffer = Vec::from(s.as_bytes());

    let _f = c.put_object(PutObjectRequest { 
        bucket,
        key,
        body: Some(StreamingBody::from(buffer)),
        ..Default::default()
    }).await?;

    // by using f we can obtain the e-tag right away! Maybe for later..

    Ok(())
}