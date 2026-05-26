use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const WS_URL: &str = "ws://172.16.173.140:3000/ws";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    #[serde(rename = "type")]
    pub kind: String,
    pub article_id: Option<u32>,
    pub article_title: Option<String>,
    pub author_name: Option<String>,
    pub preview: Option<String>,
    pub time: String,
}

pub async fn connect(tx: mpsc::Sender<Notification>) {
    let Ok((mut ws, _)) = connect_async(WS_URL).await else {
        return;
    };

    while let Some(Ok(msg)) = ws.next().await {
        if let Message::Text(text) = msg {
            if let Ok(notif) = serde_json::from_str::<Notification>(&text) {
                if tx.send(notif).await.is_err() {
                    break;
                }
            }
        }
    }
}
