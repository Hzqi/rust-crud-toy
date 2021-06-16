use std::str::FromStr;

use async_trait::async_trait;
use futures::TryStreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, Bson},
    Collection,
};

use crate::{
    entity, po,
    repository::{condition::ConditionHandler, CRUDRepository},
};

use super::{MongoDB, MongoDBConditionHandler, MongodbError};

#[derive(Clone)]
pub struct UserRepo {
    db: MongoDB,
}

impl UserRepo {
    pub fn new(db: MongoDB) -> Self {
        UserRepo { db }
    }
    fn get_collection(&self) -> Collection {
        self.db.get_collection("users")
    }
}

#[async_trait]
impl CRUDRepository<entity::User> for UserRepo {
    type Error = MongodbError;

    async fn count(
        &self,
        condition: &crate::repository::condition::Condition,
    ) -> Result<u64, Self::Error> {
        let result = self
            .get_collection()
            .count_documents(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?;
        Ok(result)
    }

    async fn exist(
        &self,
        condition: &crate::repository::condition::Condition,
    ) -> Result<bool, Self::Error> {
        let result = self
            .get_collection()
            .count_documents(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?;
        Ok(result != 0)
    }

    async fn find_one(
        &self,
        condition: &crate::repository::condition::Condition,
    ) -> Result<entity::User, Self::Error> {
        self.get_collection()
            .find_one(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?
            .ok_or(MongodbError::DataNotFoundError)
            .map(|doc| -> Result<entity::User, MongodbError> {
                let obj = bson::from_document::<po::User>(doc)?;
                let oid = match obj.id {
                    Some(i) => i.to_hex(),
                    None => String::new(),
                };
                Ok(entity::User {
                    id: oid,
                    username: obj.username,
                    passowrd_hash: String::from("******"),
                })
            })?
    }

    async fn find(
        &self,
        condition: &crate::repository::condition::Condition,
    ) -> Result<Vec<entity::User>, Self::Error> {
        let mut cursor = self
            .get_collection()
            .find(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?;
        let mut result = vec![];
        while let Some(doc) = cursor.try_next().await? {
            let obj = bson::from_document::<po::User>(doc)?;
            let oid = match obj.id {
                Some(i) => i.to_hex(),
                None => String::new(),
            };
            let user = entity::User {
                id: oid,
                username: obj.username,
                passowrd_hash: String::from("******"),
            };
            result.push(user);
        }
        return Ok(result);
    }

    async fn create(&self, data: &entity::User) -> Result<String, Self::Error> {
        let po = po::User {
            id: None,
            username: data.username.clone(),
            passowrd_hash: data.passowrd_hash.clone(),
        };
        let bson = bson::to_bson(&po)?;
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

    async fn update(&self, data: &entity::User) -> Result<bool, Self::Error> {
        let oid = ObjectId::from_str(&data.id)?;
        let po = po::User {
            id: Some(oid.clone()),
            username: data.username.clone(),
            passowrd_hash: data.passowrd_hash.clone(),
        };
        let bson = bson::to_bson(&po)?;
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

    async fn delete(
        &self,
        condition: &crate::repository::condition::Condition,
    ) -> Result<bool, Self::Error> {
        let result = self
            .get_collection()
            .delete_one(MongoDBConditionHandler::transfer_condition(condition), None)
            .await?;
        Ok(result.deleted_count == 1)
    }
}
