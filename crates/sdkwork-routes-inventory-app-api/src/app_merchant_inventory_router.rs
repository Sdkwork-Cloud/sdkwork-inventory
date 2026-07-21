use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use sdkwork_contract_service::CommerceServiceError;
use sdkwork_iam_context_service::IamAppContext;
use sdkwork_inventory_repository_sqlx::{
    MerchantInventoryScopeQuery, PostgresCommerceInventoryStore, SqliteCommerceInventoryStore,
};
use serde::Serialize;
use sqlx::{PgPool, SqlitePool};
use std::sync::Arc;

use crate::subject::app_runtime_subject_from_extension;

pub type CommerceMerchantInventoryFuture<'a, T> = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<T, CommerceServiceError>> + Send + 'a>,
>;

pub trait CommerceMerchantInventoryStore: Send + Sync {
    fn list_merchant_stocks<'a>(
        &'a self,
        scope: MerchantInventoryScopeQuery,
    ) -> CommerceMerchantInventoryFuture<'a, Vec<serde_json::Value>>;

    fn create_merchant_adjustment<'a>(
        &'a self,
        scope: MerchantInventoryScopeQuery,
        stock_id: String,
        payload: serde_json::Value,
    ) -> CommerceMerchantInventoryFuture<'a, serde_json::Value>;
}

#[derive(Clone)]
struct MerchantInventoryState {
    store: Arc<dyn CommerceMerchantInventoryStore>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MerchantInventoryApiResult<T: Serialize> {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    page: u32,
    page_size: u32,
    total: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListData<T: Serialize> {
    items: Vec<T>,
    page_info: PageInfo,
}

impl<T: Serialize> MerchantInventoryApiResult<T> {
    fn success(data: T) -> Self {
        Self {
            code: "0".into(),
            msg: "success".into(),
            data: Some(data),
        }
    }
}

pub fn app_merchant_inventory_router_with_sqlite_pool(pool: SqlitePool) -> Router {
    build_app_merchant_inventory_router(Arc::new(SqliteCommerceInventoryStore::new(pool)))
}

pub fn app_merchant_inventory_router_with_postgres_pool(pool: PgPool) -> Router {
    build_app_merchant_inventory_router(Arc::new(PostgresCommerceInventoryStore::new(pool)))
}

pub fn build_app_merchant_inventory_router(
    store: Arc<dyn CommerceMerchantInventoryStore>,
) -> Router {
    Router::new()
        .route(
            "/app/v3/api/shops/current/inventory/stocks",
            get(list_current_inventory_stocks),
        )
        .route(
            "/app/v3/api/shops/current/inventory/stocks/{stockId}/adjustments",
            post(create_current_inventory_adjustment),
        )
        .with_state(MerchantInventoryState { store })
}

impl CommerceMerchantInventoryStore for SqliteCommerceInventoryStore {
    fn list_merchant_stocks<'a>(
        &'a self,
        scope: MerchantInventoryScopeQuery,
    ) -> CommerceMerchantInventoryFuture<'a, Vec<serde_json::Value>> {
        Box::pin(async move { self.list_merchant_stocks(scope).await })
    }

    fn create_merchant_adjustment<'a>(
        &'a self,
        scope: MerchantInventoryScopeQuery,
        stock_id: String,
        payload: serde_json::Value,
    ) -> CommerceMerchantInventoryFuture<'a, serde_json::Value> {
        Box::pin(async move {
            self.create_merchant_adjustment(scope, &stock_id, payload)
                .await
        })
    }
}

impl CommerceMerchantInventoryStore for PostgresCommerceInventoryStore {
    fn list_merchant_stocks<'a>(
        &'a self,
        scope: MerchantInventoryScopeQuery,
    ) -> CommerceMerchantInventoryFuture<'a, Vec<serde_json::Value>> {
        Box::pin(async move { self.list_merchant_stocks(scope).await })
    }

    fn create_merchant_adjustment<'a>(
        &'a self,
        scope: MerchantInventoryScopeQuery,
        stock_id: String,
        payload: serde_json::Value,
    ) -> CommerceMerchantInventoryFuture<'a, serde_json::Value> {
        Box::pin(async move {
            self.create_merchant_adjustment(scope, &stock_id, payload)
                .await
        })
    }
}

async fn merchant_scope(
    runtime_context: Option<Extension<IamAppContext>>,
) -> Result<MerchantInventoryScopeQuery, Response> {
    let subject = match app_runtime_subject_from_extension(runtime_context) {
        Ok(subject) => subject,
        Err(message) => return Err(unauthorized_response(message)),
    };
    MerchantInventoryScopeQuery::new(&subject.tenant_id, subject.organization_id.as_deref())
        .map_err(|error| validation_response(error.message()))
}

async fn list_current_inventory_stocks(
    State(state): State<MerchantInventoryState>,
    runtime_context: Option<Extension<IamAppContext>>,
) -> Response {
    let scope = match merchant_scope(runtime_context).await {
        Ok(scope) => scope,
        Err(resp) => return resp,
    };
    match state.store.list_merchant_stocks(scope).await {
        Ok(items) => {
            let total = items.len() as u64;
            Json(MerchantInventoryApiResult::success(list_data(
                items, 1, 20, total,
            )))
            .into_response()
        }
        Err(error) => inventory_error_response("merchant inventory stocks list failed", error),
    }
}

async fn create_current_inventory_adjustment(
    State(state): State<MerchantInventoryState>,
    runtime_context: Option<Extension<IamAppContext>>,
    Path(stock_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let scope = match merchant_scope(runtime_context).await {
        Ok(scope) => scope,
        Err(resp) => return resp,
    };
    match state
        .store
        .create_merchant_adjustment(scope, stock_id, body)
        .await
    {
        Ok(item) => Json(MerchantInventoryApiResult::success(item)).into_response(),
        Err(error) => inventory_error_response("merchant inventory adjustment failed", error),
    }
}

fn list_data<T: Serialize>(items: Vec<T>, page: u32, page_size: u32, total: u64) -> ListData<T> {
    ListData {
        items,
        page_info: PageInfo {
            page,
            page_size,
            total,
        },
    }
}

fn unauthorized_response(message: impl Into<String>) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(MerchantInventoryApiResult::<serde_json::Value> {
            code: "401".into(),
            msg: message.into(),
            data: None,
        }),
    )
        .into_response()
}

fn validation_response(message: impl Into<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(MerchantInventoryApiResult::<serde_json::Value> {
            code: "400".into(),
            msg: message.into(),
            data: None,
        }),
    )
        .into_response()
}

fn inventory_error_response(context: &str, error: CommerceServiceError) -> Response {
    let _ = context;
    match error.code() {
        "validation" => (
            StatusCode::BAD_REQUEST,
            Json(MerchantInventoryApiResult::<serde_json::Value> {
                code: "4001".into(),
                msg: error.message().to_owned(),
                data: None,
            }),
        )
            .into_response(),
        "not_found" => (
            StatusCode::NOT_FOUND,
            Json(MerchantInventoryApiResult::<serde_json::Value> {
                code: "4040".into(),
                msg: error.message().to_owned(),
                data: None,
            }),
        )
            .into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(MerchantInventoryApiResult::<serde_json::Value> {
                code: "5000".into(),
                msg: error.message().to_owned(),
                data: None,
            }),
        )
            .into_response(),
    }
}
