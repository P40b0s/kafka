use std::fmt::Debug;
use log::{error, info, warn};
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, FromClientConfigAndContext, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{BorrowedHeaders, Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use serde::Deserialize;

pub struct CustomContext;
impl ClientContext for CustomContext {}
impl ConsumerContext for CustomContext 
{
    fn pre_rebalance(&self, rebalance: &Rebalance) 
    {
        info!("Pre rebalance {:?}", rebalance);
    }
    fn post_rebalance(&self, rebalance: &Rebalance) 
    {
        info!("Post rebalance {:?}", rebalance);
    }
    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        match result 
        {
            Ok(_) => info!("Offsets committed successfully"),
            Err(e) => warn!("Error while committing offsets: {}", e),
        };
    }
}

pub struct CustomConsumer<C: ConsumerContext>
{
    consumer: StreamConsumer<C>,
}

impl<C: ConsumerContext> CustomConsumer<C> where StreamConsumer<C>: FromClientConfigAndContext<CustomContext>
{
    pub fn new(client_id: &str, group_id: &str, topics: &[&str], brokers: &[&str]) -> Self
    {
        let context = CustomContext;
        let consumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("client.id", client_id)
            .set("bootstrap.servers", brokers.join(","))
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            //.set("statistics.interval.ms", "30000")
            //.set("auto.offset.reset", "smallest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Consumer creation failed");
        let s = Self
        {
            consumer
        };
        s.consumer
            .subscribe(&topics.to_vec())
            .expect("Can't subscribe to specified topics");
        s
    }
}

pub trait CustomConsumerTrait
{
    /// Get message from server  
    /// for work with message use closure `f`  
    fn consume<P: Debug, F>(&self, f: F) -> 
    impl std::future::Future<Output = ()> + Send + Sync 
    where for <'de> P : Deserialize<'de>,
    F: Fn(Option<P>, Option<&BorrowedHeaders>) + Send + Sync;
}

impl<C: ConsumerContext + 'static> CustomConsumerTrait for CustomConsumer<C>
{
    async fn consume<P: Debug, F>(&self, f: F) where for <'de> P : Deserialize<'de>,  F: Fn(Option<P>, Option<&BorrowedHeaders>) + Send + Sync
    {
        loop 
        {
            match self.consumer.recv().await 
            {
                Err(e) => warn!("Kafka error: {}", e),
                Ok(m) => 
                {
                    let payload = if let Some(message_payload) = m.payload()
                    {
                        if let Ok(ser) = serde_json::from_slice::<P>(message_payload)
                        {
                            Some(ser)
                        }
                        else 
                        {
                            error!("Deserialization payload error: {:?} (key: '{:?}' topic: {})", message_payload, m.key(), m.topic());
                            None
                        }
                    }
                    else
                    {
                        error!("In message key: '{:?}' topic: {} payload not found", m.key(), m.topic());
                        None
                    };
                    info!("key: '{:?}', payload: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                        m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                    if let Some(headers) = m.headers() 
                    {
                        for header in headers.iter() 
                        {
                            info!("Header {:#?}: {:?}", header.key, header.value);
                        }
                    }
                    f(payload, m.headers());
                    self.consumer.commit_message(&m, CommitMode::Async).unwrap();
                    
                }
            };
        }
    }
}