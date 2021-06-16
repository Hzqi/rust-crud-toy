use crate::repository::condition::*;

use mongodb::{
    bson::{self, Bson, Document},
    Collection,
};
use mongodb::{
    options::{ClientOptions, CountOptions, FindOptions},
    Client,
};
pub mod user;
pub mod workspace;

const APP_NAME: &str = "table-toy";
const DB_NAME: &str = "tabletoydb";

#[derive(thiserror::Error, Debug)]
pub enum MongodbError {
    #[error("data not found")]
    DataNotFoundError,
    #[error(transparent)]
    MongoDBError(#[from] mongodb::error::Error),
    #[error(transparent)]
    BsonDeError(#[from] mongodb::bson::de::Error),
    #[error(transparent)]
    BsonSerError(#[from] mongodb::bson::ser::Error),
    #[error(transparent)]
    BsonOidError(#[from] mongodb::bson::oid::Error),
}

#[derive(Clone, Debug)]
pub struct MongoDB {
    client: Client,
}

impl MongoDB {
    pub async fn init(uri: &str) -> Result<Self, MongodbError> {
        let mut client_options = ClientOptions::parse(&uri).await?;
        client_options.app_name = Some(APP_NAME.to_string());
        Ok(Self {
            client: Client::with_options(client_options)?,
        })
    }

    fn get_collection(&self, collection_name: &str) -> Collection {
        self.client.database(DB_NAME).collection(collection_name)
    }
}

/// MongoDB条件处理器
pub struct MongoDBConditionHandler {}

impl ConditionHandler for MongoDBConditionHandler {
    type TransferResult = Document;
    type TransferPageResult = (FindOptions, CountOptions);

    fn transfer_condition(condition: &Condition) -> Self::TransferResult {
        match condition {
            Condition::Empty => Document::new(),
            Condition::Single(node) => single_condition_to_doc(node),
            Condition::Complex {
                and: ands,
                or: ors,
                nor: nors,
            } => complex_condition_to_doc(ands, ors, nors),
        }
    }

    fn transfer_page_options(page_option: &PageOption) -> Self::TransferPageResult {
        let skip = (page_option.page - 1) * page_option.size;
        let find_options = FindOptions::builder()
            .skip(skip as u64)
            .limit(page_option.size as i64)
            .build();
        let count_options = CountOptions::builder()
            .skip(skip as u64)
            .limit(page_option.size as u64)
            .build();
        (find_options, count_options)
    }
}

/// 简单条件转成Document
fn single_condition_to_doc(node: &ConditionNode) -> Document {
    let mut doc = Document::new();
    doc.insert(
        &node.field,
        operate_value_to_doc(&node.operate, &node.value),
    );
    doc
}

/// 复杂条件转成Document
fn complex_condition_to_doc(
    ands: &Vec<Condition>,
    ors: &Vec<Condition>,
    nors: &Vec<Condition>,
) -> Document {
    let and_documents: Vec<Document> = ands
        .iter()
        .map(|x| MongoDBConditionHandler::transfer_condition(x))
        .collect();
    let or_documents: Vec<Document> = ors
        .iter()
        .map(|x| MongoDBConditionHandler::transfer_condition(x))
        .collect();
    let nor_documents: Vec<Document> = nors
        .iter()
        .map(|x| MongoDBConditionHandler::transfer_condition(x))
        .collect();
    let mut doc = Document::new();
    doc.insert("$and", and_documents);
    doc.insert("$or", or_documents);
    doc.insert("$nor", nor_documents);
    doc
}

/// 将判断操作符和值内容转成Document
fn operate_value_to_doc(operate: &Operate, value: &ConditionValue) -> Document {
    use ConditionValue::*;
    use Operate::*;
    let op = match operate {
        Eq => "$eq",
        Ne => "$ne",
        Lt => "$lt",
        Le => "$le",
        Gt => "$gt",
        Ge => "$ge",
        In => "$in",
        Contains => "$regex",
    };
    let val: Bson = match value {
        StringValue(v) => {
            if &op == &"$regex" {
                Bson::String(format!(".*{}.*", v))
            } else {
                Bson::String(v.to_string())
            }
        }
        StringVecValue(v) => {
            let vs = v.iter().map(|x| Bson::String(x.to_string())).collect();
            Bson::Array(vs)
        }
        Int32Value(v) => Bson::Int32(*v),
        Int32VecValue(v) => {
            let vs = v.iter().map(|x| Bson::Int32(*x)).collect();
            Bson::Array(vs)
        }
        Int64Value(v) => Bson::Int64(*v),
        Int64VecValue(v) => {
            let vs = v.iter().map(|x| Bson::Int64(*x)).collect();
            Bson::Array(vs)
        }
        BooleanValue(v) => Bson::Boolean(*v),
        DateTimeValue(v) => Bson::DateTime(bson::DateTime::from_millis(v.timestamp())),
        ObjectIdValue(v) => Bson::ObjectId(v.clone()),
    };
    let mut doc = Document::new();
    doc.insert(op, val);
    doc
}
