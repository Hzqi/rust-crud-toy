use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;

/// 描述条件的结构
#[derive(Clone, Debug)]
pub enum Condition {
    Empty,
    Single(ConditionNode),
    Complex {
        and: Box<Vec<Condition>>,
        or: Box<Vec<Condition>>,
        nor: Box<Vec<Condition>>,
    },
}

impl Condition {
    pub fn single(field: String, operate: Operate, value: ConditionValue) -> Self {
        Condition::Single(ConditionNode {
            field,
            operate,
            value,
        })
    }

    pub fn and(conditions: Vec<(String, Operate, ConditionValue)>) -> Self {
        let conds = conditions
            .iter()
            .map(|tuple| {
                let t = tuple.to_owned();
                Condition::Single(ConditionNode {
                    field: t.0,
                    operate: t.1,
                    value: t.2,
                })
            })
            .collect();
        Condition::Complex {
            and: Box::new(conds),
            or: Box::new(vec![]),
            nor: Box::new(vec![]),
        }
    }

    pub fn or(conditions: Vec<(String, Operate, ConditionValue)>) -> Self {
        let conds = conditions
            .iter()
            .map(|tuple| {
                let t = tuple.to_owned();
                Condition::Single(ConditionNode {
                    field: t.0,
                    operate: t.1,
                    value: t.2,
                })
            })
            .collect();
        Condition::Complex {
            and: Box::new(vec![]),
            or: Box::new(conds),
            nor: Box::new(vec![]),
        }
    }

    pub fn nor(conditions: Vec<(String, Operate, ConditionValue)>) -> Self {
        let conds = conditions
            .iter()
            .map(|tuple| {
                let t = tuple.to_owned();
                Condition::Single(ConditionNode {
                    field: t.0,
                    operate: t.1,
                    value: t.2,
                })
            })
            .collect();
        Condition::Complex {
            and: Box::new(vec![]),
            or: Box::new(vec![]),
            nor: Box::new(conds),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Operate {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    Contains,
}

#[derive(Clone, Debug)]
pub enum ConditionValue {
    StringValue(String),
    StringVecValue(Vec<String>),
    Int32Value(i32),
    Int32VecValue(Vec<i32>),
    Int64Value(i64),
    Int64VecValue(Vec<i64>),
    BooleanValue(bool),
    DateTimeValue(DateTime<Utc>),
    ObjectIdValue(ObjectId),
}

#[derive(Clone, Debug)]
pub struct ConditionNode {
    pub field: String,
    pub operate: Operate,
    pub value: ConditionValue,
}

/// 分页设置
/// #[derive(Clone, Debug)]
pub struct PageOption {
    pub page: usize,
    pub size: usize,
}

/// 条件转换器特型
///
/// 将描述条件的结构转成实际底层数据库交互的条件逻辑结构
pub trait ConditionHandler {
    type TransferResult;
    type TransferPageResult;

    fn transfer_condition(condition: &Condition) -> Self::TransferResult;
    fn transfer_page_options(page_option: &PageOption) -> Self::TransferPageResult;
}
