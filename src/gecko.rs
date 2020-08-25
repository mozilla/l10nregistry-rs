use std::path::Path;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

pub async fn fetch(path: &Path) -> io::Result<String> {
    println!("Fetching async for {:#?}", path);
    let mut f = File::open(path).await?;
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer).await?;

    Ok(String::from_utf8(buffer).unwrap())
}

pub fn fetch_sync(path: &Path) -> io::Result<String> {
    std::fs::read_to_string(path)
}
