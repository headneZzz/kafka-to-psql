use rdkafka::message::ToBytes;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{StreamConsumer, Consumer};
use tokio_postgres::{NoTls, Error};
use std::collections::{HashMap, VecDeque};
use futures::stream::StreamExt;
use rdkafka::{Message, Offset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct KafkaMessage {
    id: Uuid,
    name: String,
}

const BATCH_SIZE: usize = 500; // Пример размера батча

#[tokio::main]
async fn main() {
    let kafka_config = HashMap::from([
        ("bootstrap.servers", "localhost:9092,localhost:9093"),
        ("group.id", "test-group"),
        ("auto.offset.reset", "earliest")

    ]);
    let topic = "finance";
    let pg_config = "host=localhost user=user password=password dbname=fin";

    if let Err(e) = run_consumer_and_write_to_db(&kafka_config, topic, pg_config).await {
        println!("Failed to run consumer and write to db: {}", e);
    }
}

async fn run_consumer_and_write_to_db(kafka_config: &HashMap<&str, &str>, topic: &str, pg_config: &str) -> Result<(), Box<dyn std::error::Error>> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", kafka_config["bootstrap.servers"])
        .set("group.id", kafka_config["group.id"])
        .set("auto.offset.reset", kafka_config["auto.offset.reset"])
        .create()?;
    consumer.subscribe(&[topic])?;
    println!("subscribe");




    let (client, connection) = tokio_postgres::connect(pg_config, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            println!("connection error: {}", e);
        }
    });

    // let desired_offset = Offset::Beginning; // Or Offset::End to read from the end
    // consumer.seek(topic, 0, desired_offset, None).expect("Seek error");

    let mut buffer: VecDeque<KafkaMessage> = VecDeque::new();

    let mut message_stream = consumer.stream();
    println!("stream");

    while let Some(message) = message_stream.next().await {
        match message {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    let kafka_message: KafkaMessage = serde_json::from_slice(payload)?;
                    buffer.push_back(kafka_message);
                    if buffer.len() >= BATCH_SIZE {
                        write_batch_to_db(&client, &mut buffer).await?;
                    }
                }
            }
            Err(e) => println!("Kafka error: {}", e),
        }
    }

    println!("DONE");

    Ok(())
}

async fn write_batch_to_db(client: &tokio_postgres::Client, buffer: &mut VecDeque<KafkaMessage>) -> Result<(), Box<dyn std::error::Error>> {
    for message in buffer.drain(..) {
        // Запрос выполняется напрямую через клиент, без создания транзакции
        client.execute(
            "INSERT INTO users (id, name) VALUES ($1, $2)",
            &[&message.id, &message.name],
        ).await?;
    }
    Ok(())
}