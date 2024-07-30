use actix::prelude::*;
use actix_web::{web, HttpRequest, Responder};
use actix_web_actors::ws;
use crate::blockchain::chain::Chain;

struct WsSettion {
    blockchain: Chain,
}

impl Actor for WsSettion {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSettion {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => (),
            Ok(ws::Message::Text(text)) => {
                let response = self.blockchain.add_block(text.to_string());
                ctx.text(serde_json::to_string(&response).unwrap());
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

pub async fn websocket_route(req: HttpRequest, stream: web::Payload, data: web::Data<Chain>) -> impl Responder {
    ws::start(WsSettion {
        blockchain: data.get_ref().clone(),
    }, &req, stream)
}