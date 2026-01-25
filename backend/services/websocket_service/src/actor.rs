use crate::{
    models::ClientMessage,
    presence::{PresenceEntry, PresenceStore, PRESENCE_STORE},
    WebSocketSession, SESSION_STORE,
};
use actix::{ActorContext, AsyncContext};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct DocumentWsHandler {
    session_id: Uuid,
    document_id: Uuid,
    user_id: Uuid,
    display_name: String,
    color: String,
    last_heartbeat: Instant,
    presence_store: &'static PresenceStore,
    session_cleaned_up: bool, // Guard against double cleanup
}

impl DocumentWsHandler {
    pub fn new(document_id: Uuid, user_id: Uuid, display_name: String, color: String) -> Self {
        let session_id = Uuid::new_v4();

        Self {
            session_id,
            document_id,
            user_id,
            display_name,
            color,
            last_heartbeat: Instant::now(),
            presence_store: &PRESENCE_STORE,
            session_cleaned_up: false,
        }
    }

    fn start_session(&self) {
        let session = WebSocketSession::new(
            self.document_id,
            self.user_id,
            self.display_name.clone(),
            self.color.clone(),
        );
        SESSION_STORE.add_session(session);

        let entry = PresenceEntry::new(
            self.user_id,
            self.display_name.clone(),
            self.color.clone(),
            self.document_id,
        );
        self.presence_store.set_presence(entry);
    }

    fn end_session(&mut self) {
        // Guard against double cleanup - both timeout handler and stopped() may call this
        if self.session_cleaned_up {
            return;
        }
        self.session_cleaned_up = true;

        SESSION_STORE.remove_session(self.session_id);
        self.presence_store.remove_presence(self.user_id);
    }
}

impl actix::Actor for DocumentWsHandler {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_session();

        // Run heartbeat: send ping to client and check for timeout
        ctx.run_interval(HEARTBEAT_INTERVAL, |actor, ctx| {
            // Send ping to client to probe connection
            ctx.ping(&[0u8]);

            // Check if client has responded within timeout window
            if Instant::now().duration_since(actor.last_heartbeat) > CLIENT_TIMEOUT {
                tracing::warn!(
                    "WebSocket client timeout for session {} (user {})",
                    actor.session_id,
                    actor.user_id
                );
                actor.end_session();
                ctx.stop();
            }
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.end_session();
    }
}

 impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for DocumentWsHandler {
     fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
         match msg {
             Ok(ws::Message::Ping(msg)) => {
                 self.last_heartbeat = Instant::now();
                 ctx.pong(&msg);
             }
             Ok(ws::Message::Pong(_)) => {
                 self.last_heartbeat = Instant::now();
             }
             Ok(ws::Message::Text(text)) => {
                 self.last_heartbeat = Instant::now();
                 if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                     let session = WebSocketSession {
                         id: self.session_id,
                         document_id: self.document_id,
                         user_id: self.user_id,
                         display_name: self.display_name.clone(),
                         color: self.color.clone(),
                         last_activity: chrono::Utc::now(),
                     };

                     match handle_message(&session, client_msg) {
                         Ok(messages_to_send) => {
                             for msg in messages_to_send {
                                 if let Ok(json) = msg.to_json() {
                                     ctx.text(json);
                                 }
                             }
                         }
                         Err(e) => {
                             tracing::error!("Error handling WebSocket message: {}", e);
                         }
                     }
                 }
             }
             Ok(ws::Message::Binary(bin)) => {
                 self.last_heartbeat = Instant::now();
                 if let Ok(client_msg) = serde_json::from_slice::<ClientMessage>(&bin) {
                     let session = WebSocketSession {
                         id: self.session_id,
                         document_id: self.document_id,
                         user_id: self.user_id,
                         display_name: self.display_name.clone(),
                         color: self.color.clone(),
                         last_activity: chrono::Utc::now(),
                     };

                     match handle_message(&session, client_msg) {
                         Ok(messages_to_send) => {
                             for msg in messages_to_send {
                                 if let Ok(json) = msg.to_json() {
                                     ctx.text(json);
                                 }
                             }
                         }
                         Err(e) => {
                             tracing::error!("Error handling WebSocket message: {}", e);
                         }
                     }
                 }
             }
             Ok(ws::Message::Close(reason)) => {
                 ctx.close(reason);
             }
              Ok(ws::Message::Nop) => {}
              Ok(ws::Message::Continuation(_)) => {}
              Err(e) => {
                  tracing::error!("WebSocket error: {:?}", e);
              }
              }
          }

    pub async fn ws_document_handler(
    req: HttpRequest,
    stream: web::Payload,
    document_id: web::Path<Uuid>,
    user_id: web::Query<Uuid>,
    display_name: web::Query<String>,
    color: web::Query<String>,
) -> Result<HttpResponse, Error> {
    let document_id = document_id.into_inner();
    let user_id = user_id.into_inner();
    let display_name = display_name.into_inner();
    let color = color.into_inner();
    let color = if color.is_empty() { "#3B82F6".to_string() } else { color };

    let handler = DocumentWsHandler::new(document_id, user_id, display_name, color);

    let response = ws::start(handler, &req, stream)?;
    Ok(response)
}

pub async fn ws_info_handler(document_id: web::Path<Uuid>) -> actix_web::Result<HttpResponse> {
    let document_id = document_id.into_inner();
    let sessions = SESSION_STORE.get_document_sessions(document_id);

    let active_users: Vec<_> = sessions
        .iter()
        .filter_map(|session_arc| {
            let session = session_arc.lock().ok()?;
            Some(serde_json::json!({
                "session_id": session.id,
                "user_id": session.user_id,
                "display_name": session.display_name,
                "color": session.color,
                "last_activity": session.last_activity,
            }))
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "document_id": document_id,
        "active_users": active_users,
        "user_count": active_users.len(),
    })))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws/documents/{document_id}", web::get().to(ws_document_handler));
    cfg.route("/ws/documents/{document_id}/info", web::get().to(ws_info_handler));
}
