use std::time::Instant;
use actix_web_actors::ws;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use uuid::Uuid;
use crate::{
    WebSocketSession, SESSION_STORE,
    models::ClientMessage,
    presence::{PresenceStore, PresenceEntry, PRESENCE_STORE},
};

const HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

pub struct DocumentWsHandler {
    session_id: Uuid,
    document_id: Uuid,
    user_id: Uuid,
    display_name: String,
    color: String,
    last_heartbeat: Instant,
    presence_store: &'static PresenceStore,
}

impl DocumentWsHandler {
    pub fn new(
        document_id: Uuid,
        user_id: Uuid,
        display_name: String,
        color: String,
    ) -> Self {
        let session_id = Uuid::new_v4();
        
        Self {
            session_id,
            document_id,
            user_id,
            display_name,
            color,
            last_heartbeat: Instant::now(),
            presence_store: &PRESENCE_STORE,
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

    fn end_session(&self) {
        SESSION_STORE.remove_session(self.session_id);
        
        self.presence_store.remove_presence(self.user_id);
    }
}

impl actix::Actor for DocumentWsHandler {
    type Context = ws::WebsocketContext<Self>;
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
                if let Ok(_client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                self.last_heartbeat = Instant::now();
                if let Ok(_client_msg) = serde_json::from_slice::<ClientMessage>(&bin) {
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
    
    let handler = DocumentWsHandler::new(
        document_id,
        user_id,
        display_name,
        color,
    );
    
    let response = ws::start(handler, &req, stream)?;
    Ok(response)
}

pub async fn ws_info_handler(
    document_id: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
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
    cfg.route(
        "/ws/documents/{document_id}",
        web::get().to(ws_document_handler),
    );
    cfg.route(
        "/ws/documents/{document_id}/info",
        web::get().to(ws_info_handler),
    );
}
