use warp::{Rejection, Reply};

use crate::{entity, service::workspace::WorkspaceService};

use super::{
    request_object::{WorkspaceCreateParam, WorkspaceUpdateParam},
    Response,
};

/// 创建工作区
pub async fn create_workspace(
    param: WorkspaceCreateParam,
    workspace_service: WorkspaceService,
) -> Result<impl Reply, Rejection> {
    let res = workspace_service
        .create_workspace(param.name, param.description, param.creator)
        .await?;
    Response::<String> {
        success: true,
        data: res,
    }
    .to_http_reply()
}

/// 获取所有工作区
pub async fn find_all_workspace(
    workspace_service: WorkspaceService,
) -> Result<impl Reply, warp::Rejection> {
    let res = workspace_service.find_all_workspace().await?;
    Response::<Vec<entity::Workspace>> {
        success: true,
        data: res,
    }
    .to_http_reply()
}

/// 根据id获取工作区
pub async fn get_workspace_by_id(
    id: String,
    workspace_service: WorkspaceService,
) -> Result<impl Reply, Rejection> {
    let res = workspace_service.find_by_id(id).await?;
    Response::<entity::Workspace> {
        success: true,
        data: res,
    }
    .to_http_reply()
}

/// 更新工作区的名称和描述
pub async fn update_workspace_info(
    id: String,
    update_param: WorkspaceUpdateParam,
    workspace_service: WorkspaceService,
) -> Result<impl Reply, Rejection> {
    let res = workspace_service
        .update(id, update_param.name, update_param.description)
        .await?;
    Response::<()> {
        success: res,
        data: (),
    }
    .to_http_reply()
}

/// 删除工作区
pub async fn delete_workspace_by_id(
    id: String,
    workspace_service: WorkspaceService,
) -> Result<impl Reply, Rejection> {
    let res = workspace_service.delete(id).await?;
    Response::<()> {
        success: res,
        data: (),
    }
    .to_http_reply()
}
