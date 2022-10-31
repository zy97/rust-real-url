mod bilibili;

use async_trait::async_trait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyze = bilibili::Bilibili::default();
    analyze.get_real_url("6").await?;
    Ok(())
}
#[async_trait]
pub trait IAnalyze {
    async fn get_real_url(&self, room_id: &str) -> Result<(), Box<dyn std::error::Error>>;
}
