use crate::key::KeyFile;
mod key;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    let file =
        tokio::fs::read_to_string("key-94e25400-f2ce-42a0-a9b5-44973aa372b9-rietdorf_test.json")
            .await
            .unwrap();

    let key_values: KeyFile = serde_json::from_str(file.as_str())?;
    print!("{:?}", key_values);
    Ok(())
}
