//! "app" facet — API surface for the ShopFlux mobile/web app.
//!
//! All authenticated endpoints use JWT.
//! Resources are read-only projections; mutations are actions.

#[openerp_macro::facet(name = "app", module = "shop")]
pub mod app {
    // ── Resource projections ──

    /// Shop user profile.
    #[resource(path = "/me", pk = "id")]
    pub struct AppUser {
        pub id: String,
        pub username: String,
        pub display_name: Option<String>,
        pub avatar: Option<String>,
        pub role: String,
    }

    /// Product in listings.
    #[resource(path = "/products", pk = "id")]
    pub struct AppProduct {
        pub id: String,
        pub shop_id: String,
        pub shop_name: String,
        pub category_id: String,
        pub title: String,
        pub description: Option<String>,
        pub price: u64,
        pub stock: u32,
        pub images: Vec<String>,
        pub rating: u32,
        pub review_count: u32,
        pub status: String,
        pub created_at: String,
    }

    /// Category node.
    #[resource(path = "/categories", pk = "id")]
    pub struct AppCategory {
        pub id: String,
        pub name: String,
        pub parent_id: Option<String>,
        pub sort_order: u32,
    }

    /// Shop summary.
    #[resource(path = "/shops", pk = "id")]
    pub struct AppShop {
        pub id: String,
        pub owner_id: String,
        pub name: String,
        pub description: Option<String>,
        pub avatar: Option<String>,
        pub rating: u32,
        pub product_count: u32,
    }

    /// Cart entry with product snapshot.
    #[resource(path = "/cart", pk = "id")]
    pub struct AppCartItem {
        pub id: String,
        pub product_id: String,
        pub title: String,
        pub price: u64,
        pub image: Option<String>,
        pub quantity: u32,
        pub stock: u32,
    }

    /// Order overview.
    #[resource(path = "/orders", pk = "id")]
    pub struct AppOrder {
        pub id: String,
        pub shop_id: String,
        pub shop_name: String,
        pub status: String,
        pub total_amount: u64,
        pub item_count: u32,
        pub created_at: String,
    }

    /// Line item inside an order.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppOrderItem {
        pub product_id: String,
        pub title: String,
        pub price: u64,
        pub quantity: u32,
    }

    /// Order detail with line items (plain struct, not a resource).
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AppOrderDetail {
        pub id: String,
        pub shop_id: String,
        pub shop_name: String,
        pub status: String,
        pub total_amount: u64,
        pub shipping_address: String,
        pub items: Vec<AppOrderItem>,
        pub paid_at: Option<String>,
        pub created_at: String,
    }

    /// Product review.
    #[resource(path = "/reviews", pk = "id")]
    pub struct AppReview {
        pub id: String,
        pub user_id: String,
        pub username: String,
        pub rating: u32,
        pub content: String,
        pub created_at: String,
    }

    /// Shipping address.
    #[resource(path = "/addresses", pk = "id")]
    pub struct AppAddress {
        pub id: String,
        pub recipient_name: String,
        pub phone: String,
        pub province: String,
        pub city: String,
        pub district: String,
        pub detail: String,
        pub is_default: bool,
    }

    // ── Request/Response types ──

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LoginResponse {
        pub access_token: String,
        pub token_type: String,
        pub expires_in: u64,
        pub user: AppUser,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PaginationParams {
        #[serde(default = "default_limit")]
        pub limit: usize,
        #[serde(default)]
        pub offset: usize,
    }
    fn default_limit() -> usize {
        20
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ProductListResponse {
        pub items: Vec<AppProduct>,
        pub has_more: bool,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct SearchRequest {
        pub query: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ShopPageResponse {
        pub shop: AppShop,
        pub products: Vec<AppProduct>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CartResponse {
        pub items: Vec<AppCartItem>,
        pub total_amount: u64,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AddToCartRequest {
        pub product_id: String,
        pub quantity: u32,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct UpdateCartRequest {
        pub quantity: u32,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateOrderRequest {
        pub cart_item_ids: Vec<String>,
        pub address_id: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct OrderListResponse {
        pub items: Vec<AppOrder>,
        pub has_more: bool,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct PaymentResponse {
        pub success: bool,
        pub message: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct CreateReviewRequest {
        pub rating: u32,
        pub content: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReviewListResponse {
        pub items: Vec<AppReview>,
        pub has_more: bool,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateAddressRequest {
        pub recipient_name: String,
        pub phone: String,
        pub province: String,
        pub city: String,
        pub district: String,
        pub detail: String,
        pub is_default: bool,
    }

    // ── Action signatures (19 APIs) ──

    #[action(method = "POST", path = "/auth/login")]
    pub type Login = fn(req: LoginRequest) -> LoginResponse;

    #[action(method = "GET", path = "/categories")]
    pub type CategoryList = fn() -> Vec<AppCategory>;

    #[action(method = "POST", path = "/categories/{id}/products")]
    pub type CategoryProducts = fn(id: String, req: PaginationParams) -> ProductListResponse;

    #[action(method = "GET", path = "/products/{id}")]
    pub type ProductDetail = fn(id: String) -> AppProduct;

    #[action(method = "POST", path = "/search")]
    pub type ProductSearch = fn(req: SearchRequest) -> ProductListResponse;

    #[action(method = "POST", path = "/shops/{id}")]
    pub type ShopPage = fn(id: String) -> ShopPageResponse;

    #[action(method = "POST", path = "/cart")]
    pub type FetchCart = fn() -> CartResponse;

    #[action(method = "POST", path = "/cart/add")]
    pub type AddToCart = fn(req: AddToCartRequest) -> CartResponse;

    #[action(method = "PUT", path = "/cart/{id}")]
    pub type UpdateCart = fn(id: String, req: UpdateCartRequest) -> CartResponse;

    #[action(method = "POST", path = "/orders")]
    pub type CreateOrder = fn(req: CreateOrderRequest) -> AppOrderDetail;

    #[action(method = "POST", path = "/orders/list")]
    pub type OrderList = fn(req: PaginationParams) -> OrderListResponse;

    #[action(method = "GET", path = "/orders/{id}")]
    pub type OrderDetail = fn(id: String) -> AppOrderDetail;

    #[action(method = "POST", path = "/orders/{id}/pay")]
    pub type PayOrder = fn(id: String) -> PaymentResponse;

    #[action(method = "POST", path = "/products/{id}/review")]
    pub type CreateReview = fn(id: String, req: CreateReviewRequest) -> AppReview;

    #[action(method = "POST", path = "/products/{id}/reviews")]
    pub type ProductReviews = fn(id: String, req: PaginationParams) -> ReviewListResponse;

    #[action(method = "GET", path = "/addresses")]
    pub type AddressList = fn() -> Vec<AppAddress>;

    #[action(method = "POST", path = "/addresses")]
    pub type CreateAddress = fn(req: CreateAddressRequest) -> AppAddress;

    #[action(method = "PUT", path = "/addresses/{id}")]
    pub type UpdateAddress = fn(id: String, req: CreateAddressRequest) -> AppAddress;

    #[action(method = "DELETE", path = "/addresses/{id}")]
    pub type DeleteAddress = fn(id: String);
}
