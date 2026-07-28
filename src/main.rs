#[tokio::main]

async fn main() {
    let file =
        tokio::fs::read_to_string("key-94e25400-f2ce-42a0-a9b5-44973aa372b9-rietdorf_test.json")
            .await
            .unwrap();
    print!("{}", file);
}
