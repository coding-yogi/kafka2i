use consumer::Consumer;
use rdkafka::ClientConfig;


mod consumer;
mod producer;
mod cmd;
mod config;
mod metadata;

fn main() {

    let mut client_config = ClientConfig::new();
    client_config.set("bootstrap.servers", "localhost:9092");
    let consumer = match Consumer::new(&client_config){
        Ok(c) => c,
        Err(err) => {
            print!("{:?}",err);
            return;
        }
    };

    let metadata = match consumer.metadata() {
        Ok(m) => m,
        Err(err) => {
            print!("{:?}",err);
            return;
        }
    };

    let topics = metadata.topics();
    print!("{}", topics.len());

    print!("{:?}", topics[0].partitions())
}  
