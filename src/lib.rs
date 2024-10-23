mod producer;
mod consumer;
mod error;

pub use error::Error;
pub use producer::Producer;
pub use consumer::{CustomConsumer, CustomConsumerTrait};
pub use rdkafka::message::{Headers, BorrowedHeaders, ToBytes};
pub use serde::{Serialize, Deserialize};


#[cfg(test)]
mod tests 
{
    use super::consumer::{CustomConsumer, CustomConsumerTrait};
    use logger::debug;
    use super::producer::Producer;
    use super::Headers;
    use serde::Serialize;
    use serde::Deserialize;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct User
    {
        pub id: String,
        pub name: String,
        pub fio: String,
        pub old_pass: String,
        pub new_pass: String,
    }

    #[tokio::test]
    async fn test_typed_produce() 
    {
        let _ = logger::StructLogger::new_default();
        let  p = Producer::new(&["127.0.0.1:29092"], &["testing"])
        .with_header("h1", "h1_value".to_owned())
        .with_header("h2", "h2_value".to_owned());
        let user = User 
        {
            id: "123321".to_owned(),
            name: "Иван".to_owned(),
            fio: "ФИО1".to_owned(),
            old_pass: "123123".to_owned(),
            new_pass: "321321".to_owned()
        };
        let _ = p.produce("u1", &user).await;
        let _ = p.produce("u2", &user).await;
        let _ = p.produce("u3", &user).await;
    }
    #[tokio::test]
    async fn test_typed_consumer() 
    {
        let _ = logger::StructLogger::new_default();
        let  p = CustomConsumer::new("ug_client", "user_group", &["testing"], &["127.0.0.1:29092"]);
        p.consume(|p: Option<User>, h|
        {
            debug!("Поступил юзер: {}", p.unwrap().name);
        }).await;
    }

    #[tokio::test]
    ///тест одновременно двух консьюмеров в разных группах
    /// получают сообщения одновременно
    async fn test_typed_consumers() 
    {
        let _ = logger::StructLogger::new_default();
        tokio::spawn(async 
        {
            debug!("Запущена группа консьюмеров user_group");
            let  p = CustomConsumer::new("ug_client", "user_group", &["testing"], &["127.0.0.1:29092", "127.0.0.1:39092"]);
            p.consume(|p: Option<User>, h|
            {
                debug!("С 29092 Поступил юзер: {}", p.unwrap().name);
            }).await;
        });
        debug!("Запущена группа консьюмеров user_group2");
        let  p2 = CustomConsumer::new("ug_client2", "user_group2", &["testing"], &["127.0.0.1:29092", "127.0.0.1:39092"]);
        p2.consume(|p: Option<User>, h|
        {
            if let Some(header) = h
            {
                for h in header.iter()
                {

                }
            }
            debug!("С 49092 Поступил юзер: {}", p.unwrap().name);
        }).await;
    }
}
