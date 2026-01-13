use actix_web::{web, HttpResponse, Result, HttpRequest};
use uuid::Uuid;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::models::*;
use crate::repository::SpaceRepository;

const TEST_JWT_SECRET: &str = "test-secret-key-for-testing-only-do-not-use-in-production";

fn extract_user_id_from_request(req: &HttpRequest) -> Option<Uuid> {
    let auth_header = req.headers().get("authorization")?;
    let token_str = auth_header.to_str().ok()?;
    
    if !token_str.starts_with("Bearer ") {
        return None;
    }
    
    let token = &token_str[7..];
    let decoding_key = DecodingKey::from_secret(TEST_JWT_SECRET.as_bytes());
    let validation = Validation::default();
    
    match decode::<serde_json::Value>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            token_data.claims.get("sub")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
        }
        Err(_) => None,
    }
}

pub async fn list_spaces(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    let spaces = SpaceRepository::list_by_user(&pool, user_id).await
        .map_err(|e| {
            eprintln!("list_by_user error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(spaces))
}

pub async fn create_space(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    request: web::Json<CreateSpaceRequest>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    if request.name.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest("Space name cannot be empty"));
    }
    
    if request.name.len() > 200 {
        return Err(actix_web::error::ErrorBadRequest("Space name cannot exceed 200 characters"));
    }
    
    let space = SpaceRepository::create(
        &pool,
        user_id,
        &request.name,
        request.icon.clone(),
        request.description.clone(),
        request.is_public,
    ).await
        .map_err(|e| {
            eprintln!("Repository create error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Created().json(space))
}

pub async fn get_space(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    space_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    let space_id = *space_id;
    let space = SpaceRepository::find_by_id(&pool, space_id)
        .await
        .map_err(|e| {
            eprintln!("find_by_id error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Space not found"))?;
    
    let has_access = SpaceRepository::check_membership(&pool, space_id, user_id).await
        .map_err(|e| {
            eprintln!("check_membership error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    if !has_access && !space.is_public {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }
    
    Ok(HttpResponse::Ok().json(space))
}

pub async fn update_space(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    space_id: web::Path<Uuid>,
    request: web::Json<UpdateSpaceRequest>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    let space_id = *space_id;
    let space = SpaceRepository::find_by_id(&pool, space_id)
        .await
        .map_err(|e| {
            eprintln!("find_by_id error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Space not found"))?;
    
    if space.owner_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Only owner can update space"));
    }
    
    let space = SpaceRepository::update(
        &pool,
        space_id,
        request.name.clone(),
        request.icon.clone(),
        request.description.clone(),
        request.is_public,
    ).await
        .map_err(|e| {
            eprintln!("Repository update error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(space))
}

pub async fn delete_space(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    space_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    let space_id = *space_id;
    let space = SpaceRepository::find_by_id(&pool, space_id)
        .await
        .map_err(|e| {
            eprintln!("find_by_id error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Space not found"))?;
    
    if space.owner_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Only owner can delete space"));
    }
    
    SpaceRepository::delete(&pool, space_id)
        .await
        .map_err(|e| {
            eprintln!("delete error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list_space_members(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    space_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    let space_id = *space_id;
    let space = SpaceRepository::find_by_id(&pool, space_id)
        .await
        .map_err(|e| {
            eprintln!("find_by_id error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Space not found"))?;
    
    let has_access = SpaceRepository::check_membership(&pool, space_id, user_id).await
        .map_err(|e| {
            eprintln!("check_membership error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    if !has_access && space.owner_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Access denied"));
    }
    
    let members = SpaceRepository::list_members(&pool, space_id).await
        .map_err(|e| {
            eprintln!("list_members error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(members))
}

pub async fn add_space_member(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    space_id: web::Path<Uuid>,
    request: web::Json<AddMemberRequest>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    let space_id = *space_id;
    
    let space = SpaceRepository::find_by_id(&pool, space_id)
        .await
        .map_err(|e| {
            eprintln!("find_by_id error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Space not found"))?;
    
    let can_manage = SpaceRepository::check_membership(&pool, space_id, user_id).await
        .map_err(|e| {
            eprintln!("check_membership error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    if !can_manage && space.owner_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Only members can add others"));
    }
    
    let valid_roles = ["owner", "admin", "editor", "viewer"];
    if !valid_roles.contains(&request.role.as_str()) {
        return Err(actix_web::error::ErrorBadRequest("Invalid role. Must be one of: owner, admin, editor, viewer"));
    }
    
    let membership = SpaceRepository::add_member(
        &pool,
        space_id,
        &request.user_id,
        &request.role,
        &user_id.to_string(),
    ).await
        .map_err(|e| {
            eprintln!("add_member error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Created().json(membership))
}

pub async fn update_member_role(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    path: web::Path<(Uuid, Uuid)>,
    request: web::Json<UpdateMemberRequest>,
) -> Result<HttpResponse> {
    let (space_id, member_id) = path.into_inner();
    
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    let space = SpaceRepository::find_by_id(&pool, space_id)
        .await
        .map_err(|e| {
            eprintln!("find_by_id error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Space not found"))?;
    
    if space.owner_id != user_id {
        return Err(actix_web::error::ErrorForbidden("Only owner can update member roles"));
    }
    
    let membership = SpaceRepository::update_member_role(
        &pool,
        member_id,
        &request.role,
    ).await
        .map_err(|e| {
            eprintln!("update_member_role error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(membership))
}

pub async fn remove_member(
    pool: web::Data<sqlx::PgPool>,
    req: HttpRequest,
    space_id: web::Path<Uuid>,
    member_id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = match extract_user_id_from_request(&req) {
        Some(id) => id,
        None => return Err(actix_web::error::ErrorUnauthorized("Missing or invalid token")),
    };
    
    let space_id = *space_id;
    let member_id = *member_id;
    let space = SpaceRepository::find_by_id(&pool, space_id)
        .await
        .map_err(|e| {
            eprintln!("find_by_id error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Space not found"))?;
    
    let membership = SpaceRepository::get_membership(&pool, member_id)
        .await
        .map_err(|e| {
            eprintln!("get_membership error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Membership not found"))?;
    
    let can_remove = user_id == membership.invited_by || space.owner_id == user_id;
    
    if !can_remove {
        return Err(actix_web::error::ErrorForbidden("Cannot remove this member"));
    }
    
    if membership.role == "owner" {
        return Err(actix_web::error::ErrorBadRequest("Cannot remove owner"));
    }
    
    SpaceRepository::remove_member(&pool, member_id)
        .await
        .map_err(|e| {
            eprintln!("remove_member error: {:?}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::NoContent().finish())
}
