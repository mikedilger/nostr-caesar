use tonic::{transport::Server, Request, Response, Status};

use nostr_caesar::nostr_caesar_server::{NostrCaesar, NostrCaesarServer};
use nostr_caesar::{Event, Filter, Answer};

pub mod nostr_caesar {
    tonic::include_proto!("nostr_caesar");
}

#[derive(Debug, Default)]
pub struct MyNostrCaesar {}

#[tonic::async_trait]
impl NostrCaesar for MyNostrCaesar {
    async fn allow_post(
        &self,
        request: Request<Event>,
    ) -> Result<Response<Answer>, Status> {
        println!("Denying all requests. request from {:?}", request.remote_addr());
        Ok(Response::new(Answer { answer: false }))
    }

    async fn allow_filter(
        &self,
        request: Request<Filter>,
    ) -> Result<Response<Answer>, Status> {
        println!("Denying all filters. request from {:?}", request.remote_addr());
        Ok(Response::new(Answer { answer: false }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.20:50051".parse().unwrap();

    println!("NostrCaesar listening on {}", addr);

    Server::builder()
        .add_service(NostrCaesarServer::new(MyNostrCaesar { }))
        .serve(addr)
        .await?;

    Ok(())
}
