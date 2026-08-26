use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait Storable: Sized + Serialize {
    fn get_name() -> String;
    fn get_store_id(&self) -> Option<i64>;

    fn serialize(&self) -> Vec<u8> {
        postcard::to_allocvec(self).unwrap()
    }

    fn deserialize(id: i64, object: &Vec<u8>) -> Result<Box<Self>, Box<dyn std::error::Error>>;
    async fn save(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}
