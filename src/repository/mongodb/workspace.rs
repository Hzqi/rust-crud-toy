use std::str::FromStr;

use async_trait::async_trait;
use futures::stream::TryStreamExt;

use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson},
    Collection,
};

use crate::{
    entity,
    repository::{
        condition::{Condition, ConditionHandler},
        CRUDRepository,
    },
};

use super::{MongoDB, MongoDBConditionHandler, MongodbError};

/// Workspace的Repo
#[derive(Clone)]
pub struct WorkspaceRepo {
    db: MongoDB,
}

impl WorkspaceRepo {
    pub fn new(db: MongoDB) -> Self {
        WorkspaceRepo { db }
    }
    fn get_collection(&self) -> Collection {
        self.db.get_collection("workspaces")
    }
}

#[async_trait]
impl CRUDRepository<entity::Workspace> for WorkspaceRepo {
    type Error = MongodbError;

    async fn count(&self, condition: &Condition) -> Result<u64, Self::Error> {
        let result = self
            .get_collection()
            .count_documents(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?;
        Ok(result)
    }
    async fn exist(&self, condition: &Condition) -> Result<bool, Self::Error> {
        let result = self
            .get_collection()
            .count_documents(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?;
        Ok(result != 0)
    }
    async fn find_one(&self, condition: &Condition) -> Result<entity::Workspace, MongodbError> {
        self.get_collection()
            .find_one(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?
            .ok_or(MongodbError::DataNotFoundError)
            .map(|doc| -> Result<entity::Workspace, Self::Error> {
                let obj = bson::from_document::<crate::po::Workspace>(doc)?;
                let oid = match obj.id {
                    Some(i) => i.to_hex(),
                    None => String::new(),
                };
                Ok(entity::Workspace {
                    id: oid,
                    name: obj.name,
                    description: obj.description,
                    creator: entity::User {
                        id: obj.creator.to_hex(),
                        username: String::new(),
                        passowrd_hash: String::from("******"),
                    },
                    created_at: obj.created_at,
                    updated_at: obj.updated_at,
                })
            })?
    }
    async fn find(&self, condition: &Condition) -> Result<Vec<entity::Workspace>, MongodbError> {
        let mut cursor = self
            .get_collection()
            .find(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?;
        let mut result = vec![];
        while let Some(doc) = cursor.try_next().await? {
            let obj = bson::from_document::<crate::po::Workspace>(doc)?;
            let oid = match obj.id {
                Some(i) => i.to_hex(),
                None => String::new(),
            };
            let ws = entity::Workspace {
                id: oid,
                name: obj.name,
                description: obj.description,
                creator: entity::User {
                    id: obj.creator.to_hex(),
                    username: String::new(),
                    passowrd_hash: String::from("******"),
                },
                created_at: obj.created_at,
                updated_at: obj.updated_at,
            };
            result.push(ws);
        }
        Ok(result)
    }

    async fn create(&self, data: &entity::Workspace) -> Result<String, MongodbError> {
        let creator_oid = ObjectId::from_str(&data.creator.id)?;
        let po_obj = crate::po::Workspace {
            id: None,
            name: data.name.clone(),
            description: data.description.clone(),
            creator: creator_oid,
            created_at: data.created_at,
            updated_at: data.updated_at,
        };
        let bson = bson::to_bson(&po_obj)?;
        let doc = bson.as_document().unwrap();
        let insert_result = self
            .get_collection()
            .insert_one(doc.to_owned(), None)
            .await?;
        match insert_result.inserted_id {
            Bson::ObjectId(oid) => Ok(oid.to_hex()),
            _ => panic!(),
        }
    }

    async fn update(&self, data: &entity::Workspace) -> Result<bool, MongodbError> {
        let oid = ObjectId::from_str(&data.id)?;
        let creator_oid = ObjectId::from_str(&data.creator.id)?;
        let po_obj = crate::po::Workspace {
            id: Some(oid.clone()),
            name: data.name.clone(),
            description: data.description.clone(),
            creator: creator_oid,
            created_at: data.created_at,
            updated_at: data.updated_at,
        };
        let bson = bson::to_bson(&po_obj)?;
        let doc = bson.as_document().unwrap();
        let result = self
            .get_collection()
            .replace_one(
                doc! {"_id": Bson::ObjectId(oid.clone())},
                doc.to_owned(),
                None,
            )
            .await?;
        Ok(result.modified_count == 1)
    }

    async fn delete(&self, condition: &Condition) -> Result<bool, Self::Error> {
        let result = self
            .get_collection()
            .delete_one(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?;
        Ok(result.deleted_count == 1)
    }
}
