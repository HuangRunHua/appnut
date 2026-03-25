//! ShopFlux BFF — Flux state engine layer.

pub mod global;
pub mod i18n_strings;

use std::sync::Arc;

use flux_derive::flux_handlers;
use openerp_flux::StateStore;

use crate::request::*;
use crate::server::rest_app::app::{self, AppClient};
use crate::state::*;

pub struct MutableToken {
    token: tokio::sync::RwLock<Option<String>>,
}

impl Default for MutableToken {
    fn default() -> Self {
        Self::new()
    }
}

impl MutableToken {
    pub fn new() -> Self {
        Self {
            token: tokio::sync::RwLock::new(None),
        }
    }
    pub async fn set(&self, token: String) {
        *self.token.write().await = Some(token);
    }
}

#[async_trait::async_trait]
impl openerp_client::TokenSource for MutableToken {
    async fn token(&self) -> Result<Option<String>, openerp_client::ApiError> {
        Ok(self.token.read().await.clone())
    }
}

pub struct ShopBff {
    pub client: AppClient,
    pub token: Arc<MutableToken>,
    pub server_url: String,
    http: reqwest::Client,
}

impl ShopBff {
    pub fn new(base_url: &str) -> Self {
        let token = Arc::new(MutableToken::new());
        let ts: Arc<dyn openerp_client::TokenSource> = token.clone();
        Self {
            client: AppClient::new(base_url, ts),
            token,
            server_url: base_url.to_string(),
            http: reqwest::Client::new(),
        }
    }

    async fn authed_get(&self, path: &str) -> Option<serde_json::Value> {
        let token = self.token.token.read().await.clone();
        let url = format!("{}/app/shop{}", self.server_url, path);
        let mut req = self.http.get(&url);
        if let Some(t) = &token {
            req = req.header("authorization", format!("Bearer {}", t));
        }
        let resp = req.send().await.ok()?;
        resp.json().await.ok()
    }

    async fn authed_post(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Option<serde_json::Value> {
        let token = self.token.token.read().await.clone();
        let url = format!("{}/app/shop{}", self.server_url, path);
        let mut req = self.http.post(&url).json(body);
        if let Some(t) = &token {
            req = req.header("authorization", format!("Bearer {}", t));
        }
        let resp = req.send().await.ok()?;
        resp.json().await.ok()
    }
}

fn to_product_item(p: &app::AppProduct) -> ProductItem {
    ProductItem {
        id: p.id.clone(),
        title: p.title.clone(),
        price: p.price,
        image: p.images.first().cloned(),
        rating: p.rating,
        shop_name: p.shop_name.clone(),
    }
}

#[flux_handlers]
impl ShopBff {
    #[handle(LoginReq)]
    pub async fn handle_login(&self, req: &LoginReq, store: &StateStore) {
        store.set(
            AuthState::PATH,
            AuthState {
                phase: AuthPhase::Unauthenticated,
                user_id: None,
                token: None,
                role: None,
                busy: true,
                error: None,
            },
        );
        let login_req = app::LoginRequest {
            username: req.username.clone(),
            password: req.password.clone(),
        };
        match self.client.login(&login_req).await {
            Ok(resp) => {
                self.token.set(resp.access_token.clone()).await;
                store.set(
                    AuthState::PATH,
                    AuthState {
                        phase: AuthPhase::Authenticated,
                        user_id: Some(resp.user.id.clone()),
                        token: Some(resp.access_token),
                        role: Some(resp.user.role.clone()),
                        busy: false,
                        error: None,
                    },
                );
            }
            Err(_e) => {
                store.set(
                    AuthState::PATH,
                    AuthState {
                        phase: AuthPhase::Unauthenticated,
                        user_id: None,
                        token: None,
                        role: None,
                        busy: false,
                        error: Some(format!("Login failed for '{}'", req.username)),
                    },
                );
            }
        }
    }

    #[handle(LogoutReq)]
    pub async fn handle_logout(&self, _req: &LogoutReq, store: &StateStore) {
        store.set(
            AuthState::PATH,
            AuthState {
                phase: AuthPhase::Unauthenticated,
                user_id: None,
                token: None,
                role: None,
                busy: false,
                error: None,
            },
        );
        store.remove(CartState::PATH);
        store.remove(OrderListState::PATH);
    }

    #[handle(LoadCategoriesReq)]
    pub async fn handle_load_categories(&self, _req: &LoadCategoriesReq, store: &StateStore) {
        store.set(
            CategoriesState::PATH,
            CategoriesState {
                items: vec![],
                loading: true,
                error: None,
            },
        );
        if let Some(val) = self.authed_get("/categories").await {
            if let Ok(cats) = serde_json::from_value::<Vec<app::AppCategory>>(val) {
                store.set(
                    CategoriesState::PATH,
                    CategoriesState {
                        items: cats
                            .iter()
                            .map(|c| CategoryItem {
                                id: c.id.clone(),
                                name: c.name.clone(),
                                parent_id: c.parent_id.clone(),
                                sort_order: c.sort_order,
                            })
                            .collect(),
                        loading: false,
                        error: None,
                    },
                );
            }
        }
    }

    #[handle(LoadCategoryProductsReq)]
    pub async fn handle_load_category_products(
        &self,
        req: &LoadCategoryProductsReq,
        store: &StateStore,
    ) {
        store.set(
            CatalogProductsState::PATH,
            CatalogProductsState {
                category_id: req.category_id.clone(),
                items: vec![],
                loading: true,
                has_more: false,
                error: None,
            },
        );
        let params = app::PaginationParams {
            limit: 50,
            offset: 0,
        };
        let path = format!("/categories/{}/products", req.category_id);
        if let Some(val) = self.authed_post(&path, &params).await {
            if let Ok(resp) = serde_json::from_value::<app::ProductListResponse>(val) {
                store.set(
                    CatalogProductsState::PATH,
                    CatalogProductsState {
                        category_id: req.category_id.clone(),
                        items: resp.items.iter().map(to_product_item).collect(),
                        loading: false,
                        has_more: resp.has_more,
                        error: None,
                    },
                );
            }
        }
    }

    #[handle(LoadProductDetailReq)]
    pub async fn handle_load_product_detail(&self, req: &LoadProductDetailReq, store: &StateStore) {
        let path = format!("/products/{}", req.product_id);
        if let Some(val) = self.authed_get(&path).await {
            if let Ok(p) = serde_json::from_value::<app::AppProduct>(val) {
                store.set(
                    ProductDetailState::PATH,
                    ProductDetailState {
                        id: p.id.clone(),
                        title: p.title.clone(),
                        description: p.description.clone(),
                        price: p.price,
                        stock: p.stock,
                        images: p.images.clone(),
                        rating: p.rating,
                        review_count: p.review_count,
                        shop_id: p.shop_id.clone(),
                        shop_name: p.shop_name.clone(),
                        loading: false,
                    },
                );
            }
        }
    }

    #[handle(SearchProductsReq)]
    pub async fn handle_search_products(&self, req: &SearchProductsReq, store: &StateStore) {
        if req.query.is_empty() {
            store.set(
                SearchResultsState::PATH,
                SearchResultsState {
                    query: String::new(),
                    items: vec![],
                    loading: false,
                    error: None,
                },
            );
            return;
        }
        store.set(
            SearchResultsState::PATH,
            SearchResultsState {
                query: req.query.clone(),
                items: vec![],
                loading: true,
                error: None,
            },
        );
        let search_req = app::SearchRequest {
            query: req.query.clone(),
        };
        if let Some(val) = self.authed_post("/search", &search_req).await {
            if let Ok(resp) = serde_json::from_value::<app::ProductListResponse>(val) {
                store.set(
                    SearchResultsState::PATH,
                    SearchResultsState {
                        query: req.query.clone(),
                        items: resp.items.iter().map(to_product_item).collect(),
                        loading: false,
                        error: None,
                    },
                );
            }
        }
    }

    #[handle(LoadCartReq)]
    pub async fn handle_load_cart(&self, _req: &LoadCartReq, store: &StateStore) {
        store.set(
            CartState::PATH,
            CartState {
                items: vec![],
                total_amount: 0,
                loading: true,
                error: None,
            },
        );
        if let Some(val) = self.authed_post("/cart", &serde_json::json!({})).await {
            if let Ok(resp) = serde_json::from_value::<app::CartResponse>(val) {
                store.set(
                    CartState::PATH,
                    CartState {
                        items: resp
                            .items
                            .iter()
                            .map(|i| CartEntry {
                                id: i.id.clone(),
                                product_id: i.product_id.clone(),
                                title: i.title.clone(),
                                price: i.price,
                                image: i.image.clone(),
                                quantity: i.quantity,
                            })
                            .collect(),
                        total_amount: resp.total_amount,
                        loading: false,
                        error: None,
                    },
                );
            }
        }
    }

    #[handle(AddToCartReq)]
    pub async fn handle_add_to_cart(&self, req: &AddToCartReq, store: &StateStore) {
        let add_req = app::AddToCartRequest {
            product_id: req.product_id.clone(),
            quantity: req.quantity,
        };
        if let Some(val) = self.authed_post("/cart/add", &add_req).await {
            if let Ok(resp) = serde_json::from_value::<app::CartResponse>(val) {
                store.set(
                    CartState::PATH,
                    CartState {
                        items: resp
                            .items
                            .iter()
                            .map(|i| CartEntry {
                                id: i.id.clone(),
                                product_id: i.product_id.clone(),
                                title: i.title.clone(),
                                price: i.price,
                                image: i.image.clone(),
                                quantity: i.quantity,
                            })
                            .collect(),
                        total_amount: resp.total_amount,
                        loading: false,
                        error: None,
                    },
                );
            }
        }
    }

    #[handle(UpdateCartReq)]
    pub async fn handle_update_cart(&self, req: &UpdateCartReq, store: &StateStore) {
        let update = app::UpdateCartRequest {
            quantity: req.quantity,
        };
        let url = format!("{}/app/shop/cart/{}", self.server_url, req.item_id);
        let token = self.token.token.read().await.clone();
        let mut r = self.http.put(&url).json(&update);
        if let Some(t) = &token {
            r = r.header("authorization", format!("Bearer {}", t));
        }
        if let Ok(resp) = r.send().await {
            if let Ok(val) = resp.json::<app::CartResponse>().await {
                store.set(
                    CartState::PATH,
                    CartState {
                        items: val
                            .items
                            .iter()
                            .map(|i| CartEntry {
                                id: i.id.clone(),
                                product_id: i.product_id.clone(),
                                title: i.title.clone(),
                                price: i.price,
                                image: i.image.clone(),
                                quantity: i.quantity,
                            })
                            .collect(),
                        total_amount: val.total_amount,
                        loading: false,
                        error: None,
                    },
                );
            }
        }
    }

    #[handle(CreateOrderReq)]
    pub async fn handle_create_order(&self, req: &CreateOrderReq, store: &StateStore) {
        let create_req = app::CreateOrderRequest {
            cart_item_ids: req.cart_item_ids.clone(),
            address_id: req.address_id.clone(),
        };
        if let Some(val) = self.authed_post("/orders", &create_req).await {
            if let Ok(detail) = serde_json::from_value::<app::AppOrderDetail>(val) {
                store.set(
                    OrderDetailState::PATH,
                    OrderDetailState {
                        id: detail.id.clone(),
                        shop_name: detail.shop_name.clone(),
                        status: detail.status.clone(),
                        total_amount: detail.total_amount,
                        items: detail
                            .items
                            .iter()
                            .map(|i| OrderLineItem {
                                product_id: i.product_id.clone(),
                                title: i.title.clone(),
                                price: i.price,
                                quantity: i.quantity,
                            })
                            .collect(),
                        shipping_address: detail.shipping_address.clone(),
                        paid_at: detail.paid_at.clone(),
                        created_at: detail.created_at.clone(),
                        loading: false,
                    },
                );
            }
        }
    }

    #[handle(LoadOrderListReq)]
    pub async fn handle_load_order_list(&self, _req: &LoadOrderListReq, store: &StateStore) {
        store.set(
            OrderListState::PATH,
            OrderListState {
                items: vec![],
                loading: true,
                has_more: false,
                error: None,
            },
        );
        let params = app::PaginationParams {
            limit: 50,
            offset: 0,
        };
        if let Some(val) = self.authed_post("/orders/list", &params).await {
            if let Ok(resp) = serde_json::from_value::<app::OrderListResponse>(val) {
                store.set(
                    OrderListState::PATH,
                    OrderListState {
                        items: resp
                            .items
                            .iter()
                            .map(|o| OrderSummary {
                                id: o.id.clone(),
                                shop_name: o.shop_name.clone(),
                                status: o.status.clone(),
                                total_amount: o.total_amount,
                                item_count: o.item_count,
                                created_at: o.created_at.clone(),
                            })
                            .collect(),
                        loading: false,
                        has_more: resp.has_more,
                        error: None,
                    },
                );
            }
        }
    }

    #[handle(LoadOrderDetailReq)]
    pub async fn handle_load_order_detail(&self, req: &LoadOrderDetailReq, store: &StateStore) {
        let path = format!("/orders/{}", req.order_id);
        if let Some(val) = self.authed_get(&path).await {
            if let Ok(detail) = serde_json::from_value::<app::AppOrderDetail>(val) {
                store.set(
                    OrderDetailState::PATH,
                    OrderDetailState {
                        id: detail.id.clone(),
                        shop_name: detail.shop_name.clone(),
                        status: detail.status.clone(),
                        total_amount: detail.total_amount,
                        items: detail
                            .items
                            .iter()
                            .map(|i| OrderLineItem {
                                product_id: i.product_id.clone(),
                                title: i.title.clone(),
                                price: i.price,
                                quantity: i.quantity,
                            })
                            .collect(),
                        shipping_address: detail.shipping_address.clone(),
                        paid_at: detail.paid_at.clone(),
                        created_at: detail.created_at.clone(),
                        loading: false,
                    },
                );
            }
        }
    }

    #[handle(PayOrderReq)]
    pub async fn handle_pay_order(&self, req: &PayOrderReq, store: &StateStore) {
        let path = format!("/orders/{}/pay", req.order_id);
        if let Some(val) = self.authed_post(&path, &serde_json::json!({})).await {
            if let Ok(result) = serde_json::from_value::<app::PaymentResponse>(val) {
                if result.success {
                    let detail_path = format!("/orders/{}", req.order_id);
                    if let Some(dval) = self.authed_get(&detail_path).await {
                        if let Ok(detail) = serde_json::from_value::<app::AppOrderDetail>(dval) {
                            store.set(
                                OrderDetailState::PATH,
                                OrderDetailState {
                                    id: detail.id.clone(),
                                    shop_name: detail.shop_name.clone(),
                                    status: detail.status.clone(),
                                    total_amount: detail.total_amount,
                                    items: detail
                                        .items
                                        .iter()
                                        .map(|i| OrderLineItem {
                                            product_id: i.product_id.clone(),
                                            title: i.title.clone(),
                                            price: i.price,
                                            quantity: i.quantity,
                                        })
                                        .collect(),
                                    shipping_address: detail.shipping_address.clone(),
                                    paid_at: detail.paid_at.clone(),
                                    created_at: detail.created_at.clone(),
                                    loading: false,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    #[handle(LoadAddressListReq)]
    pub async fn handle_load_addresses(&self, _req: &LoadAddressListReq, store: &StateStore) {
        store.set(
            AddressListState::PATH,
            AddressListState {
                items: vec![],
                loading: true,
            },
        );
        if let Some(val) = self.authed_get("/addresses").await {
            if let Ok(addrs) = serde_json::from_value::<Vec<app::AppAddress>>(val) {
                store.set(
                    AddressListState::PATH,
                    AddressListState {
                        items: addrs
                            .iter()
                            .map(|a| AddressEntry {
                                id: a.id.clone(),
                                recipient_name: a.recipient_name.clone(),
                                phone: a.phone.clone(),
                                full_address: format!(
                                    "{}{}{} {}",
                                    a.province, a.city, a.district, a.detail
                                ),
                                is_default: a.is_default,
                            })
                            .collect(),
                        loading: false,
                    },
                );
            }
        }
    }
}
