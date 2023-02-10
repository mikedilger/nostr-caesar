use nostr_caesar::nostr_caesar_client::NostrCaesarClient;
use nostr_caesar::{RpcEventCheck, RpcFilterCheck};
use nostr_types::{
    Event, EventKind, Filter, PreEvent, PrivateKey, PublicKey, PublicKeyHex, Tag, Unixtime,
};

const EXAMPLE_OWNER_PUBKEY: &str =
    "00000b6b73abad367cd1924ab6700d381582a585ddc40c25df2b2be6d737488f";
const EXAMPLE_OWNER_PRVKEY: &str =
    "93342b31aee9d0597bdf9d851ab8ea6b724c83752dac8de4e4c90bee0054ebec";

const EXAMPLE_OTHER_PUBKEY: &str =
    "f332a54f2bd94988dce7f56d3df5845c2a301455c3bdd387b5c2ff0f8ca1c55a";
const EXAMPLE_OTHER_PRVKEY: &str =
    "cbfa9fb8b54ec27e570da7e27a454aec5f8550f61dcc71d7a932967b3d53455c";

pub mod nostr_caesar {
    tonic::include_proto!("nostr_caesar");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = NostrCaesarClient::connect("http://127.0.0.20:50051").await?;

    let owner_private_key = PrivateKey::try_from_hex_string(EXAMPLE_OWNER_PRVKEY)?;
    let other_private_key = PrivateKey::try_from_hex_string(EXAMPLE_OTHER_PRVKEY)?;
    let owner_pubkeyhex = PublicKeyHex::try_from_str(EXAMPLE_OWNER_PUBKEY)?;
    let other_pubkeyhex = PublicKeyHex::try_from_str(EXAMPLE_OTHER_PUBKEY)?;

    // ---------------------------------------------------------
    // Test the author's events pass
    let pre_event = PreEvent {
        pubkey: PublicKey::try_from_hex_string(EXAMPLE_OWNER_PUBKEY)?,
        created_at: Unixtime::now()?,
        kind: EventKind::TextNote,
        tags: vec![],
        content: "This is a top level post going into my outbox".to_owned(),
        ots: None,
    };
    let event = Event::new(pre_event, &owner_private_key)?;
    let request = tonic::Request::new(RpcEventCheck {
        serialized: serde_json::to_string(&event)?,
        ipaddr: "127.0.0.10".to_owned(),
        authed_pubkey: "".to_owned(),
    });
    let response = client.allow_post(request).await?;
    assert!(response.get_ref().pass);

    // ---------------------------------------------------------
    // Test someone else's events fail
    let pre_event = PreEvent {
        pubkey: PublicKey::try_from_hex_string(EXAMPLE_OTHER_PUBKEY)?,
        created_at: Unixtime::now()?,
        kind: EventKind::TextNote,
        tags: vec![],
        content: "This is someone who is not authorized to post here".to_owned(),
        ots: None,
    };
    let event = Event::new(pre_event, &other_private_key)?;
    let request = tonic::Request::new(RpcEventCheck {
        serialized: serde_json::to_string(&event)?,
        ipaddr: "127.0.0.10".to_owned(),
        authed_pubkey: "".to_owned(),
    });
    let response = client.allow_post(request).await?;
    assert!(!response.get_ref().pass);

    // ---------------------------------------------------------
    // Test someone elses kind-10002 events pass
    let pre_event = PreEvent {
        pubkey: PublicKey::try_from_hex_string(EXAMPLE_OTHER_PUBKEY)?,
        created_at: Unixtime::now()?,
        kind: EventKind::RelayList,
        tags: vec![], // no relays, this doesn't really matter
        content: "".to_owned(),
        ots: None,
    };
    let event = Event::new(pre_event, &other_private_key)?;
    let request = tonic::Request::new(RpcEventCheck {
        serialized: serde_json::to_string(&event)?,
        ipaddr: "127.0.0.10".to_owned(),
        authed_pubkey: "".to_owned(),
    });
    let response = client.allow_post(request).await?;
    assert!(response.get_ref().pass);

    // ---------------------------------------------------------
    // Test someone elses events tagging me pass
    let pre_event = PreEvent {
        pubkey: PublicKey::try_from_hex_string(EXAMPLE_OTHER_PUBKEY)?,
        created_at: Unixtime::now()?,
        kind: EventKind::RelayList,
        tags: vec![Tag::Pubkey {
            pubkey: owner_pubkeyhex.clone(),
            recommended_relay_url: None,
            petname: None,
        }],
        content: "Tag, you're it!".to_owned(),
        ots: None,
    };
    let event = Event::new(pre_event, &other_private_key)?;
    let request = tonic::Request::new(RpcEventCheck {
        serialized: serde_json::to_string(&event)?,
        ipaddr: "127.0.0.10".to_owned(),
        authed_pubkey: "".to_owned(),
    });
    let response = client.allow_post(request).await?;
    assert!(response.get_ref().pass);

    // ---------------------------------------------------------
    // Test kind 10002 Filters pass
    let filter = Filter {
        kinds: vec![EventKind::RelayList],
        ..Default::default()
    };
    let request = tonic::Request::new(RpcFilterCheck {
        serialized: serde_json::to_string(&filter)?,
        ipaddr: "127.0.0.10".to_owned(),
        authed_pubkey: "".to_owned(),
        filter_name: "general_feed".to_owned(),
    });
    let response = client.allow_filter(request).await?;
    assert!(response.get_ref().pass);

    // ---------------------------------------------------------
    // Test owner-author Filters pass
    let filter = Filter {
        authors: vec![owner_pubkeyhex.prefix(20)],
        ..Default::default()
    };
    let request = tonic::Request::new(RpcFilterCheck {
        serialized: serde_json::to_string(&filter)?,
        ipaddr: "127.0.0.10".to_owned(),
        authed_pubkey: "".to_owned(),
        filter_name: "general_feed".to_owned(),
    });
    let response = client.allow_filter(request).await?;
    assert!(response.get_ref().pass);

    // ---------------------------------------------------------
    // Test other-kind Filters fail
    let filter = Filter {
        kinds: vec![EventKind::Reaction],
        ..Default::default()
    };
    let request = tonic::Request::new(RpcFilterCheck {
        serialized: serde_json::to_string(&filter)?,
        ipaddr: "127.0.0.10".to_owned(),
        authed_pubkey: "".to_owned(),
        filter_name: "general_feed".to_owned(),
    });
    let response = client.allow_filter(request).await?;
    assert!(!response.get_ref().pass);

    // ---------------------------------------------------------
    // Test other-author Filters fail
    let filter = Filter {
        authors: vec![other_pubkeyhex.prefix(20)],
        ..Default::default()
    };
    let request = tonic::Request::new(RpcFilterCheck {
        serialized: serde_json::to_string(&filter)?,
        ipaddr: "127.0.0.10".to_owned(),
        authed_pubkey: "".to_owned(),
        filter_name: "general_feed".to_owned(),
    });
    let response = client.allow_filter(request).await?;
    assert!(!response.get_ref().pass);

    println!("All tests passed.");

    Ok(())
}
