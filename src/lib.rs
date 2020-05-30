extern crate nalgebra_glm as glm;

pub mod config;
mod rendering;

use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_serde::formats::SymmetricalBincode;
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

use embercore::*;

use crate::config::Config;

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let socket = TcpStream::connect(config.server_address).await.unwrap();

    let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());

    let mut serialized =
        tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalBincode::<protocol::Message>::default());

    serialized
        .send(protocol::Message {
            timestamp: chrono::Utc::now().timestamp(),
            test_text: "".to_string(),
        })
        .await
        .unwrap();

    Ok(())
}
