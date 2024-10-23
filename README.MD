# Simple producer and consumer for Kafka based on rdkafka
Produser:  
```rust
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