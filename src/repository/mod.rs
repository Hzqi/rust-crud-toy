pub mod condition;
pub mod mongodb;
use async_trait::async_trait;
use condition::*;

#[async_trait]
pub trait CRUDRepository<T> {
    type Error;

    /// 按条件查询数量
    async fn count(&self, condition: &Condition) -> Result<u64, Self::Error>;
    /// 按条件查询是否存在
    async fn exist(&self, condition: &Condition) -> Result<bool, Self::Error>;
    /// 按条件查询单个
    async fn find_one(&self, condition: &Condition) -> Result<T, Self::Error>;
    /// 按条件查询
    async fn find(&self, condition: &Condition) -> Result<Vec<T>, Self::Error>;
    /// 创建数据
    async fn create(&self, data: &T) -> Result<String, Self::Error>;
    /// 更想你数据
    async fn update(&self, data: &T) -> Result<bool, Self::Error>;
    /// 删除数据
    async fn delete(&self, condition: &Condition) -> Result<bool, Self::Error>;
}

pub struct PageResult<T> {
    pub datas: Vec<T>,
    pub count: i64,
}

#[async_trait]
pub trait PaginationRepository<T>: CRUDRepository<T> {
    async fn find_page(
        &self,
        condition: &Condition,
        page_setting: &PageOption,
        is_count_all: bool,
    ) -> Result<PageResult<T>, Self::Error>;
}
