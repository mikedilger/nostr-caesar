use nostr_caesar::nostr_caesar_client::NostrCaesarClient;
use nostr_caesar::{Event, Filter};

pub mod nostr_caesar {
    tonic::include_proto!("nostr_caesar");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = NostrCaesarClient::connect("http://127.0.0.20:50051").await?;

    let request = tonic::Request::new(Event {
        serialized: "SERIALIZED_EVENT".into(),
    });
    let response = client.allow_post(request).await?;
    println!("RESPONSE={:?}", response);

    let request = tonic::Request::new(Filter {
        serialized: "SERIALIZED_FILTER".into(),
    });
    let response = client.allow_filter(request).await?;
    println!("RESPONSE={:?}", response);

    Ok(())
}
