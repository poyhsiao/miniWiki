use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Space, SpaceMembership};

#[cfg(test)]
mod repository_tests;

pub struct SpaceRepository;
