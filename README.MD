# Simple producer and consumer for Kafka based on rdkafka
Producer:  
```rust
use kafka::{Serialize, Deserialize, consumer::{CustomConsumer, CustomConsumerTrait}, producer::Producer, Headers};
#[derive(Debug, Serialize, Deserialize)]
pub struct User
{
    pub id: String,
    pub name: String,
    pub fio: String,
    pub old_pass: String,
    pub new_pass: String,
}

let  p = Producer::new(&["127.0.0.1:29092"], &["testing"])
.with_header("h1", "h1_value".to_owned())
.with_header("h2", "h2_value".to_owned());
let user = User 
{
    id: "123321".to_owned(),
    name: "Юрий".to_owned(),
    fio: "ФИО1".to_owned(),
    old_pass: "123123".to_owned(),
    new_pass: "321321".to_owned()
};
p.produce("u1", &user).await;
p.produce("u2", &user).await;
p.produce("u3", &user).await;
```  
Consumer:  
```rust
let  p2 = CustomConsumer::new("user_client", "user_group", &["testing"], &["127.0.0.1:29092", "127.0.0.1:39092"]);
// manual set type for payload
p2.consume(|p: Option<User>, h|
{
    if let Some(header) = h
    {
        for h in header.iter()
        {
            todo!();
        }
    }
    true
}).await;
```  

# Share volumes

### Create directory for Kafka data  
``` bash
mkdir -p /vol1/kafka-data
```

### Make sure the user has read and write permissions.  
```bash
chown -R 1000:1000 /vol1/kafka-data
```

### Start the containers:  
``` bash
docker run -d \
    --name=kafka-vols \
    --net=host \
    --user=1000 \
    -e KAFKA_NODE_ID=1 \
    -e KAFKA_LISTENER_SECURITY_PROTOCOL_MAP='CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT,PLAINTEXT_HOST:PLAINTEXT' \
    -e KAFKA_ADVERTISED_LISTENERS='PLAINTEXT://kafka-vols:29092,PLAINTEXT_HOST://localhost:9092' \
    -e KAFKA_PROCESS_ROLES='broker,controller' \
    -e KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1 \
    -e KAFKA_CONTROLLER_QUORUM_VOTERS='1@kafka-vols:29093' \
    -e KAFKA_LISTENERS='PLAINTEXT://kafka-vols:29092,CONTROLLER://kafka-vols:29093,PLAINTEXT_HOST://0.0.0.0:9092' \
    -e KAFKA_INTER_BROKER_LISTENER_NAME='PLAINTEXT' \
    -e KAFKA_CONTROLLER_LISTENER_NAMES='CONTROLLER' \
    -e KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1 \
    -e CLUSTER_ID='MkU3OEVBNTcwNTJENDM2Qk' \
    -v /vol1/kafka-data:/var/lib/kafka/data \
    confluentinc/cp-kafka:7.7.1
```  
#### Example for find dirs for share
```bash
docker inspect kafka|jq '.[].Mounts[] | .Type ,.Destination'
"volume"
"/etc/zookeeper/secrets"
"bind"
"/var/lib/zookeeper"
"volume"
"/var/lib/zookeeper/log"
"volume"
"/var/lib/zookeeper/data"
```
