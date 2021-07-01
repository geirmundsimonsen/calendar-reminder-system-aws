use dynamodb::{Client, model::AttributeValue};
use lambda_runtime::Error;

pub async fn get_value_from_cache(key: String) -> Result<Option<String>, Error> {
    let client = Client::from_env();
    let resp = client.get_item().table_name("Cache").key("key", AttributeValue::S(key)).send().await?;

    match resp.item {
        Some(map) => {
            Ok(Some(map
                .get("value").expect("Existing key in cache must have the attr. 'value'")
                .as_s().expect("All simple keys in cache should be of the type string.")
                .clone()))
        },
        None => Ok(None)
    }
}

pub async fn store_value_in_cache(key: String, value: String) -> Result<(), Error> {
    let client = Client::from_env();
    client.put_item().table_name("Cache")
        .item("key", AttributeValue::S(key))
        .item("value", AttributeValue::S(value))
        .send().await?;

    Ok(())
}