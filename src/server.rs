use nostr_caesar::nostr_caesar_server::{NostrCaesar, NostrCaesarServer};
use nostr_caesar::{RpcAnswer, RpcEventCheck, RpcFilterCheck};
use nostr_types::{Event, EventKind, Filter, PublicKeyHex, Tag};
use tonic::{transport::Server, Request, Response, Status};

const EXAMPLE_PUBKEY: &str = "00000b6b73abad367cd1924ab6700d381582a585ddc40c25df2b2be6d737488f";

const EVENT_FAIL_MESSAGE: & str = "This relay only accepts events from it's owner, unless they are addressed to the owner, or are kind-10002. See our NIP-11 for details.";

const FILTER_FAIL_MESSAGE: & str = "This relay only serves events from it's owner and/or kind-10002 events for everybody. See our NIP-11 for details.";

pub mod nostr_caesar {
    tonic::include_proto!("nostr_caesar");
}

#[derive(Debug)]
pub struct MyNostrCaesar {
    owner_pubkey: PublicKeyHex,
}

#[tonic::async_trait]
impl NostrCaesar for MyNostrCaesar {
    async fn allow_post(
        &self,
        request: Request<RpcEventCheck>,
    ) -> Result<Response<RpcAnswer>, Status> {
        let event: Event = serde_json::from_str(&request.get_ref().serialized)
            .map_err(|e| Status::invalid_argument(format!("{e}")))?;

        // Allow if I am AUTHed (so I have full access)
        // TBD

        // Allow kind-10002 for everybody
        if event.kind == EventKind::RelayList {
            return Ok(Response::new(RpcAnswer {
                pass: true,
                message: "".to_owned(),
            }));
        }

        // Allow if I am the author (so I can push into my outbox)
        let event_pubkey: PublicKeyHex = event.pubkey.into();
        if event_pubkey == self.owner_pubkey {
            return Ok(Response::new(RpcAnswer {
                pass: true,
                message: "".to_owned(),
            }));
        }

        // Allow if I am tagged (so the world can push into my inbox)
        if event.tags.iter().any(|t| {
            if let Tag::Pubkey { pubkey, .. } = t {
                *pubkey == self.owner_pubkey
            } else {
                false
            }
        }) {
            return Ok(Response::new(RpcAnswer {
                pass: true,
                message: "".to_owned(),
            }));
        }

        println!("Denying EVENT: {}", &request.get_ref().serialized);
        Ok(Response::new(RpcAnswer {
            pass: false,
            message: EVENT_FAIL_MESSAGE.to_owned(),
        }))
    }

    async fn allow_filter(
        &self,
        request: Request<RpcFilterCheck>,
    ) -> Result<Response<RpcAnswer>, Status> {
        let filter: Filter = serde_json::from_str(&request.get_ref().serialized)
            .map_err(|e| Status::invalid_argument(format!("{e}")))?;

        // Allow if I am AUTHed (so I have full access)
        // TBD

        // Allow if strictly kind-10002 for everybody
        if filter.kinds == vec![EventKind::RelayList] {
            return Ok(Response::new(RpcAnswer {
                pass: true,
                message: "".to_owned(),
            }));
        }

        // Allow if I am the only author in the filter (so the world can pull from my outbox)
        if filter.authors.len() == 1 && filter.authors[0].matches(&self.owner_pubkey) {
            return Ok(Response::new(RpcAnswer {
                pass: true,
                message: "".to_owned(),
            }));
        }

        println!("Denying REQ Filter: {}", &request.get_ref().serialized);
        Ok(Response::new(RpcAnswer {
            pass: false,
            message: FILTER_FAIL_MESSAGE.to_owned(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.20:50051".parse().unwrap();
    println!("NostrCaesar listening on {addr}");

    let instance = MyNostrCaesar {
        owner_pubkey: PublicKeyHex::try_from_str(EXAMPLE_PUBKEY)?,
    };

    Server::builder()
        .add_service(NostrCaesarServer::new(instance))
        .serve(addr)
        .await?;

    Ok(())
}
