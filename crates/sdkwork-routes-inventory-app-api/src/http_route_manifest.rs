use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};

const HTTP_ROUTES: &[HttpRoute] = &[
    HttpRoute::dual_token(
        HttpMethod::Get,
        "/app/v3/api/shops/current/inventory/stocks",
        "commerce.inventory.read",
        "inventory.stocks.list",
    ),
    HttpRoute::dual_token(
        HttpMethod::Post,
        "/app/v3/api/shops/current/inventory/stocks/{stockId}/adjustments",
        "commerce.inventory.manage",
        "inventory.stocks.adjustments.create",
    ),
];

pub fn app_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(HTTP_ROUTES)
}
