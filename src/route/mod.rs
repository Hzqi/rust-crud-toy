use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{convert::Infallible, net::SocketAddr};
use warp::{http::StatusCode, Filter, Rejection, Reply};

use crate::{
    env_var,
    repository::mongodb::{workspace::WorkspaceRepo, MongoDB, MongodbError},
    route::request_object::WorkspaceUpdateParam,
    service::{workspace::WorkspaceService, ServiceError},
};

use self::request_object::WorkspaceCreateParam;

mod request_object;
mod workspace;

#[derive(Serialize, Deserialize, Debug)]
struct Response<T: Serialize> {
    success: bool,
    data: T,
}
impl<T: Serialize> Response<T> {
    fn to_http_reply(&self) -> Result<impl Reply, Rejection> {
        let json = warp::reply::json(&self);
        Ok(warp::reply::with_status(json, StatusCode::OK))
    }
}

impl warp::reject::Reject for ServiceError {}

/// 处理warp的不成功情况
async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    use ServiceError::*;
    let code = if err.is_not_found() {
        StatusCode::NOT_FOUND
    } else if let Some(err) = err.find::<ServiceError>() {
        match err {
            RepositoryError(e) => match e {
                MongodbError::DataNotFoundError => {
                    log::info!("data not found");
                    StatusCode::NOT_FOUND
                }
                MongodbError::BsonSerError(_) | MongodbError::BsonOidError(_) => {
                    log::info!("serialize data error");
                    StatusCode::BAD_REQUEST
                }
                _ => {
                    log::error!("{:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            },
            IoError(e) => {
                log::error!("{:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            WarpError(e) => {
                log::error!("{:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            HttpError(e) => {
                log::error!("{:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        log::warn!("{:?}", err);
        StatusCode::METHOD_NOT_ALLOWED
    } else {
        log::error!("{:?}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    };
    let json = warp::reply::json(&Response::<()> {
        success: false,
        data: (),
    });
    Ok(warp::reply::with_status(json, code))
}

fn json_body_request<T: DeserializeOwned + Send>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 64).and(warp::body::json())
}

fn with_workspace_service(
    service: WorkspaceService,
) -> impl Filter<Extract = (WorkspaceService,), Error = Infallible> + Clone {
    warp::any().map(move || service.clone())
}

/// 启动路由
pub async fn run(addr: SocketAddr) {
    let uri = env_var!("MONGODB_URI");
    let db = MongoDB::init(&uri)
        .await
        .expect("couldn't connect to MongoDB server");

    let workspace_service = WorkspaceService::new(WorkspaceRepo::new(db.clone()));

    // POST /workspaces
    let create_workspace_route = warp::path!("workspaces")
        .and(warp::post())
        .and(json_body_request::<WorkspaceCreateParam>())
        .and(with_workspace_service(workspace_service.clone()))
        .and_then(workspace::create_workspace);

    // GET /workspaces
    let get_all_workspace_route = warp::path!("workspaces")
        .and(warp::get())
        .and(with_workspace_service(workspace_service.clone()))
        .and_then(workspace::find_all_workspace);

    // GET /workspaces/:ID
    let get_workspace_route = warp::path!("workspaces" / String)
        .and(warp::get())
        .and(with_workspace_service(workspace_service.clone()))
        .and_then(workspace::get_workspace_by_id);

    // PUT /workspaces/:ID
    let update_workspace_route = warp::path!("workspaces" / String)
        .and(warp::put())
        .and(json_body_request::<WorkspaceUpdateParam>())
        .and(with_workspace_service(workspace_service.clone()))
        .and_then(workspace::update_workspace_info);

    // DELETE /workspaces/:ID
    let delete_workspace_route = warp::path!("workspaces" / String)
        .and(warp::delete())
        .and(with_workspace_service(workspace_service))
        .and_then(workspace::delete_workspace_by_id);

    // 返回整个route
    let routes = warp::path("api")
        .and(
            create_workspace_route
                .or(get_all_workspace_route)
                .or(get_workspace_route)
                .or(update_workspace_route)
                .or(delete_workspace_route),
        )
        .recover(handle_rejection)
        .with(warp::log("crud-toy"));
    warp::serve(routes).run(addr).await
}
