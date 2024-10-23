use std::marker::PhantomData;
use std::time::Duration;
use log::info;
use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;


pub struct Producer<P: Serialize>
{
    topics: Vec<String>,
    producer: FutureProducer,
    headers: Option<OwnedHeaders>,
    _phantom_data: PhantomData<P>

}
impl<P: Serialize> Producer<P>
{
    ///brokers in format x.x.x.x:xxxx
    pub fn new(brokers: &[&str], topics: &[&str]) -> Self
    {
        Self 
        {
            topics: topics.iter().map(|t| t.to_string()).collect(),
            producer: ClientConfig::new()
            .set("bootstrap.servers", brokers.join(","))
            .set("message.timeout.ms", "5000")
            .set("acks", "1")
            .create()
            .expect("Producer creation error"),
            headers: None,
            _phantom_data: PhantomData::default()
        }
    }
    pub fn with_header<V: rdkafka::message::ToBytes>(mut self, key: &str, value: V) -> Self
    {
        if self.headers.is_some()
        {
            let added = self.headers.unwrap().clone().insert(Header
            {
                key,
                value: Some(&value),
            });
            self.headers = Some(added);
        }
        else 
        {
            self.headers = Some(OwnedHeaders::new().insert(Header 
            {
                key,
                value: Some(&value),
            }));
        }
        self
    }
    pub async fn produce(&self, key: &str, payload: &P) -> Result<(), super::error::Error>
    {
        let payload = serde_json::to_vec(payload)?;
        let delivery_status = futures::future::try_join_all(self.topics.iter().map(|output_topic| 
        {
            let mut record = FutureRecord::to(output_topic);
            record = record.payload(&payload);
            record = record.key(key);
            if let Some(h) = self.headers.clone()
            {
                record = record.headers(h);
            }
            self.producer.send(record, Duration::from_secs(0))
        }))
        .await;
        info!("Delivery status for message {} received", key);
        info!("Future completed. Result: {:?}", delivery_status);
        Ok(())
    }
}