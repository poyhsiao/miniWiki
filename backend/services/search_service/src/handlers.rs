use actix_web::{web, Responder, HttpResponse};
use tracing::{info, error};
use crate::models::*;
use crate::repository::{SearchRepository, SearchRepositoryTrait};
use shared_errors::AppError;
use validator::Validate;

// Helper for user extraction - in real implementation, this would come from JWT
fn extract_user_id(req: &actix_web::HttpRequest) -> Result<String, AppError> {
    req.headers()
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::AuthenticationError("Missing X-User-Id header".to_string()))
}

// Search documents endpoint
pub async fn search_documents(
    query: web::Query<SearchQuery>,
    repo: web::Data<SearchRepository>,
    http_req: actix_web::HttpRequest,
) -> impl Responder {
    let start_time = std::time::Instant::now();

    // Validate request
    if let Err(validation_errors) = (*query).validate() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<()>::error("VALIDATION_ERROR", &format!("Validation failed: {:?}", validation_errors)));
    }

    let user_id = match extract_user_id(&http_req) {
        Ok(id) => id,
        Err(e) => return HttpResponse::Unauthorized().json(ApiResponse::<()>::error("UNAUTHORIZED", &e.to_string())),
    };

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    let query_length = query.q.len();
    info!("Search initiated (query_length={}, limit={}, offset={})", query_length, limit, offset);

    match repo.search(&user_id, &query.q, query.space_id.as_deref(), limit, offset).await {
        Ok((results, total)) => {
            let elapsed_ms = start_time.elapsed().as_millis() as i64;
            info!("Search completed in {}ms, found {} results", elapsed_ms, total);

            HttpResponse::Ok()
                .json(ApiResponse::<SearchResponse>::success(SearchResponse {
                    results: results.into_iter().map(|r| SearchResult {
                        document_id: r.document_id.to_string(),
                        space_id: r.space_id.to_string(),
                        space_name: r.space_name,
                        title: r.title,
                        snippet: r.content.as_str().unwrap_or("").to_string(),
                        score: r.score,
                    }).collect(),
                    total,
                    took: elapsed_ms,
                }))
        }
        Err(e) => {
            error!("Search error: {:?}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("SEARCH_ERROR", "Search failed. Please try again later."))
        }
    }
}
