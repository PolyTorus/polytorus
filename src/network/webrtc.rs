use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::{
    api::{media_engine::MediaEngine, APIBuilder, API},
    data_channel::{data_channel_init::RTCDataChannelInit, RTCDataChannel},
    ice_transport::{ice_connection_state::RTCIceConnectionState, ice_server::RTCIceServer},
    peer_connection::{configuration::RTCConfiguration, RTCPeerConnection},
};

// 型安全な設定構造体
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebRTCConfig {
    pub host: String,
    pub port: u16,
    pub stun_url: String,
    pub turn_config: Option<TurnConfig>,
    pub peer_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub port: u16,
}

// WebRTCの状態を表現するenum
#[derive(Debug, Clone, PartialEq)]
pub enum WebRTCState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

pub struct WebRTCNode {
    config: WebRTCConfig,
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Arc<RTCDataChannel>,
    state: Arc<Mutex<WebRTCState>>,
}

impl WebRTCNode {
    pub async fn new(config: WebRTCConfig) -> Result<Self, Box<dyn Error>> {
        let mut rtc_config = RTCConfiguration::default();

        // STUNサーバーの設定
        rtc_config.ice_servers.push(RTCIceServer {
            urls: vec![config.stun_url.clone()],
            ..Default::default()
        });

        // TURNサーバーの設定（存在する場合）
        if let Some(turn_config) = &config.turn_config {
            rtc_config.ice_servers.push(RTCIceServer {
                urls: vec![turn_config.url.clone()],
                username: turn_config.username.clone(),
                credential: turn_config.password.clone(),
                ..Default::default()
            });
        }

        // MediaEngineの設定
        let mut m = MediaEngine::default();
        let api = APIBuilder::new().with_media_engine(m).build();

        // PeerConnectionの作成
        let peer_connection = Arc::new(api.new_peer_connection(rtc_config).await?);

        // DataChannelの設定
        let data_channel = peer_connection
            .create_data_channel(
                "blockchain",
                Some(RTCDataChannelInit {
                    ordered: Some(true),
                    ..Default::default()
                }),
            )
            .await?;

        let node = WebRTCNode {
            config,
            peer_connection: peer_connection.clone(),
            data_channel: data_channel.clone(),
            state: Arc::new(Mutex::new(WebRTCState::New)),
        };

        // イベントハンドラーの設定
        node.setup_event_handlers().await?;

        Ok(node)
    }

    async fn setup_event_handlers(&self) -> Result<(), Box<dyn Error>> {
        let pc = self.peer_connection.clone();
        let state = self.state.clone();

        // ICE接続状態の監視
        pc.on_ice_connection_state_change(Box::new(move |s: RTCIceConnectionState| {
            let state_clone = state.clone();
            Box::pin(async move {
                let new_state = match s {
                    RTCIceConnectionState::Connected => WebRTCState::Connected,
                    RTCIceConnectionState::Disconnected => WebRTCState::Disconnected,
                    RTCIceConnectionState::Failed => WebRTCState::Failed,
                    RTCIceConnectionState::Closed => WebRTCState::Closed,
                    _ => return,
                };

                let mut current_state = state_clone.lock().await;
                *current_state = new_state;
            })
        }));

        // データチャンネルのイベントハンドラー
        let dc = self.data_channel.clone();
        dc.on_open(Box::new(move || {
            Box::pin(async move {
                println!("Data channel opened");
            })
        }));

        dc.on_message(Box::new(move |msg| {
            Box::pin(async move {
                println!("Message received: {:?}", msg);
                // TODO: メッセージ処理ロジックをここに実装
            })
        }));

        Ok(())
    }

    pub async fn connect(&self) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.lock().await;
        *state = WebRTCState::Connecting;

        // TODO: 接続処理の実装
        // シグナリングサーバーとの通信やSDPの交換など

        Ok(())
    }

    pub async fn send_message(&self, message: &[u8]) -> Result<(), Box<dyn Error>> {
        let bytes = Bytes::copy_from_slice(message);
        self.data_channel.send(&bytes).await?;
        Ok(())
    }

    pub async fn get_state(&self) -> WebRTCState {
        self.state.lock().await.clone()
    }

    // TODO: ピアの発見
    async fn discover_peers(&self) -> Vec<PeerInfo> {
        // ブロックチェーンからアクティブなピア情報を取得
        // STUNサーバー情報も分散管理可能
        vec![]
    }

    // TODO: 接続の確立
    async fn establish_connection(&self, peer: PeerInfo) {
        // ブロックチェーン経由でシグナリング情報を交換
        // SDPの交換をスマートコントラクト経由で実施
    }
}

// Drop実装で適切なリソース解放を保証
impl Drop for WebRTCNode {
    fn drop(&mut self) {
        // TODO: クリーンアップ処理
    }
}
