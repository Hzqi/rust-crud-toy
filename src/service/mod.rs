use crate::repository::mongodb::MongodbError;

pub mod workspace;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    RepositoryError(#[from] MongodbError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    WarpError(#[from] warp::Error),
    #[error(transparent)]
    HttpError(#[from] warp::http::Error),
}

impl From<mongodb::bson::oid::Error> for ServiceError {
    fn from(err: mongodb::bson::oid::Error) -> Self {
        ServiceError::RepositoryError(MongodbError::BsonOidError(err))
    }
}
