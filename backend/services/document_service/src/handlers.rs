use actix_web::web;

pub async fn list_documents() -> impl actix_web::Responder {
    actix_web::web::Json(serde_json::json!({
        "documents": []
    }))
}

pub async fn create_document(req: web::Json<serde_json::Value>) -> impl actix_web::Responder {
    actix_web::web::Json(serde_json::json!({
        "id": "new-document-id",
        "message": "Document created"
    }))
}
