mod data;
use std::{fmt::format, sync::Arc, time::Duration};

use actix::{clock::Instant, Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use data::{AppData, XWsSub};
use serde_json::Value;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SayHello(pub String);

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);
/// Define HTTP actor
struct MyWs {
    data: Arc<AppData>,
    hb: Instant,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
    /// 连接上
    fn started(&mut self, ws: &mut Self::Context) {
        println!("{:?} join!", ws.address());
    }

    /// 断开连接
    fn stopped(&mut self, ws: &mut Self::Context) {
        println!("{:?} exit!", ws.address());
    }
}

impl MyWs {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }

    fn ws_text(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let req: Value = serde_json::from_str(text)?;
        let key = "op";
        let target_value = req.get(key).ok_or(format!("{} key not found", key))?;
        println!("{:?}", target_value);
        let a = serde_json::from_value(target_value.clone()).unwrap();
        println!("{:?}", a);
        Ok(a)
    }

    fn chuli(&mut self, ctx: &mut <Self as Actor>::Context, text: String) {
        let data = self.data.clone();
        let addr = ctx.address();
        if let Ok(a) = self.ws_text(ctx, &text) {
            match &a as &str {
                "a" => AppData::push_to_client(&data, "1", "22222".to_string()),
                "b" => AppData::push_to_client(&data, "2", "23333".to_string()),
                "c" => ctx.text("cccccccccccccccccc"),
                "sub" => {
                    if let Ok(sub) = serde_json::from_str::<XWsSub>(&text) {
                        self.data.client_sub(addr.recipient(), sub);
                    }
                }
                _ => {}
            }
        };
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("text");
                self.chuli(ctx, text.to_string());
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl Handler<SayHello> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: SayHello, ctx: &mut Self::Context) {
        let a = format!("aaaa {}", msg.0);
        ctx.text(a);
    }
}

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppData>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        MyWs {
            data: (*data).clone(),
            hb: Instant::now(),
        },
        &req,
        stream,
    );
    println!("{:?}", req);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(AppData::new());
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/ws", web::get().to(index))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
