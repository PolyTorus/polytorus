use crate::Result;
use webrtc::*;

#[derive(Clone, Debug)]
pub struct WebRTCConfig {
    pub host: String,
    pub port: String,
    pub stun_url: String,
    pub turn_url: Option<String>,
    pub turn_username: Option<String>,
    pub turn_password: Option<String>,
    pub peer_id: Option<String>,
}

pub struct WebRTCNode {
    config: WebRTCConfig,
    peer_connection: RTCPeerConnection,
    data_channel: RTCDataChannel,
}

impl WebRTCNode {
    pub fn new(config: WebRTCConfig) -> Result<Self> {
        // WebRTC設定の初期化
        let mut rtc_config = RTCConfiguration::default();

        // STUNサーバーの設定
        rtc_config.ice_servers.push(RTCIceServer {
            urls: vec![config.stun_url.clone()],
            ..Default::default()
        });

        // TURNサーバーの設定
        if let Some(turn_url) = &config.turn_url {
            rtc_config.ice_servers.push(RTCIceServer {
                urls: vec![turn_url.clone()],
                username: config.turn_username.clone(),
                credential: config.turn_password.clone(),
                ..Default::default()
            });
        }

        // PeerConnectionの作成
        let peer_connection = RTCPeerConnection::new(&rtc_config)?;

        // DataChannelの設定
        let data_channel = peer_connection.create_data_channel("blockchain", Some(RTCDataChannelInit {
            ordered: Some(true),
            ..Default::default()
        }))?;

        Ok(WebRTCNode {
            config,
            peer_connection,
            data_channel,
        })
    }

    pub fn start (&mut self) -> Result<()> {
        // シグナリングサーバーとの接続処理
        // P2P接続の確率
        // メッセージングの処理
        // など、WebRTCの具体的な実装

        println!("WebRTC node started on {}:{}", self.Ok())
    }
}