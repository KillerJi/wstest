use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

use actix::Recipient;
use serde::{Deserialize, Serialize};

use crate::SayHello;
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct XWsSub {
    pub target: String,
}

pub struct AppData {
    client_list: RwLock<HashMap<Recipient<SayHello>, HashSet<XWsSub>>>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            client_list: RwLock::new(HashMap::new()),
        }
    }

    pub fn push_to_client(&self, target: &str, data: String) {
        if let Ok(client_list) = self.client_list.read() {
            for (client, sub_set) in client_list.iter() {
                for sub in sub_set {
                    if sub.target != target {
                        continue;
                    }
                    client.do_send(SayHello(data.clone()));
                }
            }
        }
    }

    pub fn client_sub(
        &self,
        recipient: Recipient<SayHello>,
        sub: XWsSub,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut client_list) = self.client_list.write() {
            client_list
                .entry(recipient)
                .or_insert_with(HashSet::new)
                .insert(sub);
            Ok(())
        } else {
            Err("unknown error".into())
        }
    }
}
