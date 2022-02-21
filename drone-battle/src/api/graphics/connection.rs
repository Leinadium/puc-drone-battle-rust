extern crate paho_mqtt as mqtt;

use std::time::Duration;
use mqtt::{Client, Message, CreateOptionsBuilder, ConnectOptionsBuilder};
use serde_json;
use crate::api::graphics::data::Data;

pub struct Connection {
    mqtt_id: String,
    client: Option<Client>
}

impl Connection {
    pub fn new(identifier: String) -> Connection {
        Connection {
            mqtt_id: format!("bot-{}", identifier),
            client: None,
        }
    }

    pub fn connect(&mut self) -> Option<()> {
        let create_ops = CreateOptionsBuilder::new()
            .server_uri("tcp://localhost:1883")
            .client_id(self.mqtt_id.clone())
            .persistence(None)
            .finalize();

        let cli= Client::new(create_ops).ok()?;

        let connect_ops = ConnectOptionsBuilder::new()
            .connect_timeout(Duration::from_secs(1))
            .finalize();

        if let Err(e) = cli.connect(connect_ops) {
            println!("[CONNECTION] unable to connect: {:?}", e);
            return None;
        }

        self.client = Some(cli);
        Some(())
    }

    pub fn send(&mut self, data: Data) -> Option<()> {
        if self.client.is_none() {
            println!("[CONNECTION] client is not connected. skipping data");
            return None;
        }

        let payload = serde_json::to_string(&data).ok()?;
        let msg = Message::new("puc-drone-battle-rust", payload, 0);
        self.client.as_ref().unwrap().publish(msg).ok()?;
        Some(())
    }

    pub fn disconnect(self) {
        self.client.unwrap().disconnect(None).ok();
    }
}