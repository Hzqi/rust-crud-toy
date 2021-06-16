use std::str::FromStr;

use chrono::Utc;
use mongodb::bson::oid::ObjectId;

use crate::{
    entity::{self, User},
    repository::{
        condition::{Condition, ConditionValue, Operate},
        mongodb::workspace::WorkspaceRepo,
        CRUDRepository,
    },
};

use super::ServiceError;

#[derive(Clone)]
pub struct WorkspaceService {
    repo: WorkspaceRepo,
}

impl WorkspaceService {
    pub fn new(repo: WorkspaceRepo) -> Self {
        Self { repo }
    }

    pub async fn create_workspace(
        &self,
        name: String,
        description: String,
        creator: String,
    ) -> Result<String, ServiceError> {
        let now = Utc::now();
        let result = self
            .repo
            .create(&entity::Workspace {
                id: String::new(),
                name,
                description,
                creator: User {
                    id: creator,
                    username: String::new(),
                    passowrd_hash: String::new(),
                },
                created_at: now.clone(),
                updated_at: now.clone(),
            })
            .await?;
        Ok(result)
    }

    pub async fn find_all_workspace(&self) -> Result<Vec<entity::Workspace>, ServiceError> {
        let result = self.repo.find(&Condition::Empty).await?;
        Ok(result)
    }

    pub async fn find_by_id(&self, id: String) -> Result<entity::Workspace, ServiceError> {
        let oid = ObjectId::from_str(&id)?;
        let result = self
            .repo
            .find_one(&Condition::single(
                String::from("_id"),
                Operate::Eq,
                ConditionValue::ObjectIdValue(oid),
            ))
            .await?;
        Ok(result)
    }

    pub async fn update(
        &self,
        id: String,
        name: String,
        description: String,
    ) -> Result<bool, ServiceError> {
        let oid = ObjectId::from_str(&id)?;
        let mut workspace = self
            .repo
            .find_one(&Condition::single(
                String::from("_id"),
                Operate::Eq,
                ConditionValue::ObjectIdValue(oid),
            ))
            .await?;
        workspace.name = name;
        workspace.description = description;
        workspace.updated_at = Utc::now();
        let result = self.repo.update(&workspace).await?;
        Ok(result)
    }

    pub async fn delete(&self, id: String) -> Result<bool, ServiceError> {
        let oid = ObjectId::from_str(&id)?;
        let result = self
            .repo
            .delete(&Condition::single(
                String::from("_id"),
                Operate::Eq,
                ConditionValue::ObjectIdValue(oid),
            ))
            .await?;
        Ok(result)
    }
}
