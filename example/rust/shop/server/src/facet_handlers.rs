//! Facet handler implementations for the ShopFlux "app" facet.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

use openerp_core::ServiceError;
use openerp_store::KvOps;
use openerp_types::*;

use crate::server::i18n::Localizer;
use crate::server::jwt::JwtService;
use crate::server::model::*;
use crate::server::payment::PaymentProvider;
use crate::server::rest_app::app::{self, *};

pub struct FacetStateInner {
    pub users: KvOps<User>,
    pub shops: KvOps<Shop>,
    pub categories: KvOps<Category>,
    pub products: KvOps<Product>,
    pub cart_items: KvOps<CartItem>,
    pub orders: KvOps<Order>,
    pub order_items: KvOps<OrderItem>,
    pub reviews: KvOps<Review>,
    pub addresses: KvOps<Address>,
    pub jwt: JwtService,
    pub i18n: Box<dyn Localizer>,
    pub payment: Box<dyn PaymentProvider>,
}

pub type FacetState = Arc<FacetStateInner>;

fn current_user(headers: &HeaderMap, state: &FacetStateInner) -> Result<String, ServiceError> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| ServiceError::Unauthorized(state.i18n.t("error.auth.missing_token", &[])))?;
    let claims = state
        .jwt
        .verify(token)
        .map_err(|_| ServiceError::Unauthorized(state.i18n.t("error.auth.invalid_token", &[])))?;
    Ok(claims.sub)
}

fn lang_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("accept-language")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("en")
        .split(',')
        .next()
        .unwrap_or("en")
        .trim()
        .to_string()
}

fn parse_images(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn to_app_product(p: &Product, state: &FacetStateInner) -> AppProduct {
    let shop = state.shops.get(p.shop.resource_id()).ok().flatten();
    AppProduct {
        id: p.id.to_string(),
        shop_id: p.shop.resource_id().to_string(),
        shop_name: shop.map(|s| s.name.clone()).unwrap_or_default(),
        category_id: p.category.resource_id().to_string(),
        title: p.title.clone(),
        description: p.product_description.clone(),
        price: p.price,
        stock: p.stock,
        images: parse_images(&p.images),
        rating: p.rating,
        review_count: p.review_count,
        status: p.status.clone(),
        created_at: p.created_at.to_string(),
    }
}

fn to_app_user(u: &User) -> AppUser {
    AppUser {
        id: u.id.to_string(),
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        avatar: u.avatar.as_ref().map(|a| a.to_string()),
        role: u.role.clone(),
    }
}

fn to_app_category(c: &Category, lang: &str) -> AppCategory {
    AppCategory {
        id: c.id.to_string(),
        name: c.cat_name.get(lang).to_string(),
        parent_id: c.parent.as_ref().map(|n| n.resource_id().to_string()),
        sort_order: c.sort_order,
    }
}

fn to_app_address(a: &Address) -> AppAddress {
    AppAddress {
        id: a.id.to_string(),
        recipient_name: a.recipient_name.clone(),
        phone: a.phone.clone(),
        province: a.province.clone(),
        city: a.city.clone(),
        district: a.district.clone(),
        detail: a.detail.clone(),
        is_default: a.is_default,
    }
}

fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

// ── E-02-01: Login ──

pub async fn login(
    _headers: HeaderMap,
    State(state): State<FacetState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ServiceError> {
    let user = state
        .users
        .get(&req.username)
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or_else(|| {
            ServiceError::Unauthorized(
                state
                    .i18n
                    .t("error.auth.user_not_found", &[("username", &req.username)]),
            )
        })?;
    if let Some(ref stored_hash) = user.password_hash
        && !verify_password(&req.password, stored_hash.as_str())
    {
        return Err(ServiceError::Unauthorized(
            state.i18n.t("error.auth.invalid_token", &[]),
        ));
    }
    let display = user.display_name.as_deref().unwrap_or(&user.username);
    let token = state
        .jwt
        .issue(user.id.as_str(), display, &user.role)
        .map_err(ServiceError::Internal)?;
    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: 86400,
        user: to_app_user(&user),
    }))
}

// ── E-02-02: Category list ──

pub async fn category_list(
    headers: HeaderMap,
    State(state): State<FacetState>,
) -> Result<Json<Vec<AppCategory>>, ServiceError> {
    let lang = lang_from_headers(&headers);
    let mut cats = state
        .categories
        .list()
        .map_err(|e| ServiceError::Internal(e.to_string()))?;
    cats.sort_by_key(|c| c.sort_order);
    Ok(Json(
        cats.iter().map(|c| to_app_category(c, &lang)).collect(),
    ))
}

// ── E-02-03: Category products ──

pub async fn category_products(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(cat_id): Path<String>,
    Json(params): Json<PaginationParams>,
) -> Result<Json<ProductListResponse>, ServiceError> {
    let _uid = current_user(&headers, &state)?;
    let all = state.products.list().unwrap_or_default();
    let filtered: Vec<AppProduct> = all
        .iter()
        .filter(|p| p.category.resource_id() == cat_id && p.status == "on_sale")
        .map(|p| to_app_product(p, &state))
        .collect();
    let total = filtered.len();
    let offset = params.offset.min(total);
    let end = (offset + params.limit).min(total);
    Ok(Json(ProductListResponse {
        items: filtered[offset..end].to_vec(),
        has_more: end < total,
    }))
}

// ── E-02-04: Product detail ──

pub async fn product_detail(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(id): Path<String>,
) -> Result<Json<AppProduct>, ServiceError> {
    let _uid = current_user(&headers, &state)?;
    let p = state
        .products
        .get(&id)
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or_else(|| {
            ServiceError::NotFound(state.i18n.t("error.product.not_found", &[("id", &id)]))
        })?;
    Ok(Json(to_app_product(&p, &state)))
}

// ── E-02-05: Product search ──

pub async fn product_search(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<ProductListResponse>, ServiceError> {
    let _uid = current_user(&headers, &state)?;
    let q = req.query.to_lowercase();
    let items: Vec<AppProduct> = state
        .products
        .list()
        .unwrap_or_default()
        .iter()
        .filter(|p| {
            p.status == "on_sale"
                && (p.title.to_lowercase().contains(&q)
                    || p.product_description
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q))
        })
        .map(|p| to_app_product(p, &state))
        .collect();
    Ok(Json(ProductListResponse {
        has_more: false,
        items,
    }))
}

// ── E-02-06: Shop page ──

pub async fn shop_page(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(id): Path<String>,
) -> Result<Json<ShopPageResponse>, ServiceError> {
    let _uid = current_user(&headers, &state)?;
    let s = state
        .shops
        .get(&id)
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or_else(|| {
            ServiceError::NotFound(state.i18n.t("error.shop.not_found", &[("id", &id)]))
        })?;
    let products: Vec<AppProduct> = state
        .products
        .list()
        .unwrap_or_default()
        .iter()
        .filter(|p| p.shop.resource_id() == id && p.status == "on_sale")
        .map(|p| to_app_product(p, &state))
        .collect();
    Ok(Json(ShopPageResponse {
        shop: AppShop {
            id: s.id.to_string(),
            owner_id: s.owner.resource_id().to_string(),
            name: s.name.clone(),
            description: s.shop_description.clone(),
            avatar: s.avatar.as_ref().map(|a| a.to_string()),
            rating: s.rating,
            product_count: s.product_count,
        },
        products,
    }))
}

// ── E-02-07: Get cart ──

fn build_cart(uid: &str, state: &FacetStateInner) -> CartResponse {
    let items: Vec<AppCartItem> = state
        .cart_items
        .list()
        .unwrap_or_default()
        .iter()
        .filter(|ci| ci.user.resource_id() == uid)
        .filter_map(|ci| {
            let p = state.products.get(ci.product.resource_id()).ok()??;
            let imgs = parse_images(&p.images);
            Some(AppCartItem {
                id: ci.id.to_string(),
                product_id: p.id.to_string(),
                title: p.title.clone(),
                price: p.price,
                image: imgs.first().cloned(),
                quantity: ci.quantity,
                stock: p.stock,
            })
        })
        .collect();
    let total = items.iter().map(|i| i.price * i.quantity as u64).sum();
    CartResponse {
        items,
        total_amount: total,
    }
}

pub async fn get_cart(
    headers: HeaderMap,
    State(state): State<FacetState>,
) -> Result<Json<CartResponse>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    Ok(Json(build_cart(&uid, &state)))
}

// ── E-02-08: Add to cart ──

pub async fn add_to_cart(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Json(req): Json<AddToCartRequest>,
) -> Result<Json<CartResponse>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    let _product = state
        .products
        .get(&req.product_id)
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or_else(|| {
            ServiceError::NotFound(
                state
                    .i18n
                    .t("error.product.not_found", &[("id", &req.product_id)]),
            )
        })?;
    let cart_key = format!("{}:{}", uid, req.product_id);
    if let Ok(Some(mut existing)) = state.cart_items.get(&cart_key) {
        existing.quantity += req.quantity;
        let _ = state.cart_items.save(existing);
    } else {
        let _ = state.cart_items.save_new(CartItem {
            id: Id::default(),
            user: Name::new(format!("shop/users/{}", uid)),
            product: Name::new(format!("shop/products/{}", req.product_id)),
            quantity: req.quantity,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        });
    }
    Ok(Json(build_cart(&uid, &state)))
}

// ── E-02-09: Update cart quantity ──

pub async fn update_cart(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(item_id): Path<String>,
    Json(req): Json<UpdateCartRequest>,
) -> Result<Json<CartResponse>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    if req.quantity == 0 {
        let _ = state.cart_items.delete(&item_id);
    } else if let Ok(Some(mut ci)) = state.cart_items.get(&item_id) {
        ci.quantity = req.quantity;
        let _ = state.cart_items.save(ci);
    }
    Ok(Json(build_cart(&uid, &state)))
}

// ── E-02-10: Create order ──

pub async fn create_order(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<AppOrderDetail>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    if req.cart_item_ids.is_empty() {
        return Err(ServiceError::Validation(
            state.i18n.t("error.cart.empty_selection", &[]),
        ));
    }

    let addr = state
        .addresses
        .get(&req.address_id)
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or_else(|| {
            ServiceError::NotFound(
                state
                    .i18n
                    .t("error.address.not_found", &[("id", &req.address_id)]),
            )
        })?;

    let addr_json = serde_json::json!({
        "name": addr.recipient_name,
        "phone": addr.phone,
        "province": addr.province,
        "city": addr.city,
        "district": addr.district,
        "detail": addr.detail,
    })
    .to_string();

    let mut total: u64 = 0;
    let mut order_items_data = Vec::new();
    let mut shop_id: Option<String> = None;

    for ci_id in &req.cart_item_ids {
        let ci = state
            .cart_items
            .get(ci_id)
            .map_err(|e| ServiceError::Internal(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound(format!("Cart item '{}' not found", ci_id)))?;
        let product = state
            .products
            .get(ci.product.resource_id())
            .map_err(|e| ServiceError::Internal(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Product not found".into()))?;

        if product.stock < ci.quantity {
            return Err(ServiceError::Validation(
                state
                    .i18n
                    .t("error.stock.insufficient", &[("title", &product.title)]),
            ));
        }

        let sid = product.shop.resource_id().to_string();
        if let Some(ref existing) = shop_id {
            if *existing != sid {
                return Err(ServiceError::Validation(
                    "All items must be from the same shop".into(),
                ));
            }
        } else {
            shop_id = Some(sid.clone());
        }

        total += product.price * ci.quantity as u64;
        order_items_data.push((ci.clone(), product.clone()));
    }

    let shop_id = shop_id.unwrap();
    let snapshot: Vec<serde_json::Value> = order_items_data
        .iter()
        .map(|(ci, p)| {
            serde_json::json!({
                "product_id": p.id.to_string(),
                "title": p.title,
                "price": p.price,
                "quantity": ci.quantity,
            })
        })
        .collect();

    let order = state
        .orders
        .save_new(Order {
            id: Id::default(),
            buyer: Name::new(format!("shop/users/{}", uid)),
            shop: Name::new(format!("shop/shops/{}", shop_id)),
            status: "pending_payment".into(),
            total_amount: total,
            shipping_address: addr_json,
            items_snapshot: serde_json::to_string(&snapshot).unwrap_or_default(),
            paid_at: None,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

    let mut app_items = Vec::new();
    for (ci, product) in &order_items_data {
        let _ = state.order_items.save_new(OrderItem {
            id: Id::default(),
            order: Name::new(format!("shop/orders/{}", order.id)),
            product: Name::new(format!("shop/products/{}", product.id)),
            title_snapshot: product.title.clone(),
            price_snapshot: product.price,
            quantity: ci.quantity,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        });

        if let Ok(Some(mut p)) = state.products.get(product.id.as_str()) {
            p.stock = p.stock.saturating_sub(ci.quantity);
            let _ = state.products.save(p);
        }

        app_items.push(AppOrderItem {
            product_id: product.id.to_string(),
            title: product.title.clone(),
            price: product.price,
            quantity: ci.quantity,
        });
    }

    for ci_id in &req.cart_item_ids {
        let _ = state.cart_items.delete(ci_id);
    }

    let shop = state.shops.get(&shop_id).ok().flatten();
    Ok(Json(AppOrderDetail {
        id: order.id.to_string(),
        shop_id,
        shop_name: shop.map(|s| s.name.clone()).unwrap_or_default(),
        status: order.status.clone(),
        total_amount: order.total_amount,
        shipping_address: order.shipping_address.clone(),
        items: app_items,
        paid_at: order.paid_at.clone(),
        created_at: order.created_at.to_string(),
    }))
}

// ── E-02-11: Order list ──

pub async fn order_list(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Json(params): Json<PaginationParams>,
) -> Result<Json<OrderListResponse>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    let all_orders = state.orders.list().unwrap_or_default();
    let mut orders: Vec<&Order> = all_orders
        .iter()
        .filter(|o| o.buyer.resource_id() == uid)
        .collect();
    orders.sort_by(|a, b| b.created_at.as_str().cmp(a.created_at.as_str()));
    let total = orders.len();
    let offset = params.offset.min(total);
    let end = (offset + params.limit).min(total);
    let items: Vec<AppOrder> = orders[offset..end]
        .iter()
        .map(|o| {
            let shop = state.shops.get(o.shop.resource_id()).ok().flatten();
            let item_count: usize = state
                .order_items
                .list()
                .unwrap_or_default()
                .iter()
                .filter(|oi| oi.order.resource_id() == o.id.as_str())
                .count();
            AppOrder {
                id: o.id.to_string(),
                shop_id: o.shop.resource_id().to_string(),
                shop_name: shop.map(|s| s.name.clone()).unwrap_or_default(),
                status: o.status.clone(),
                total_amount: o.total_amount,
                item_count: item_count as u32,
                created_at: o.created_at.to_string(),
            }
        })
        .collect();
    Ok(Json(OrderListResponse {
        items,
        has_more: end < total,
    }))
}

// ── E-02-12: Order detail ──

pub async fn order_detail(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(id): Path<String>,
) -> Result<Json<AppOrderDetail>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    let order = state
        .orders
        .get(&id)
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or_else(|| {
            ServiceError::NotFound(state.i18n.t("error.order.not_found", &[("id", &id)]))
        })?;
    if order.buyer.resource_id() != uid {
        return Err(ServiceError::NotFound(
            state.i18n.t("error.order.not_found", &[("id", &id)]),
        ));
    }
    let items: Vec<AppOrderItem> = state
        .order_items
        .list()
        .unwrap_or_default()
        .iter()
        .filter(|oi| oi.order.resource_id() == id)
        .map(|oi| AppOrderItem {
            product_id: oi.product.resource_id().to_string(),
            title: oi.title_snapshot.clone(),
            price: oi.price_snapshot,
            quantity: oi.quantity,
        })
        .collect();
    let shop = state.shops.get(order.shop.resource_id()).ok().flatten();
    Ok(Json(AppOrderDetail {
        id: order.id.to_string(),
        shop_id: order.shop.resource_id().to_string(),
        shop_name: shop.map(|s| s.name.clone()).unwrap_or_default(),
        status: order.status.clone(),
        total_amount: order.total_amount,
        shipping_address: order.shipping_address.clone(),
        items,
        paid_at: order.paid_at.clone(),
        created_at: order.created_at.to_string(),
    }))
}

// ── E-02-13: Pay order ──

pub async fn pay_order(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(id): Path<String>,
) -> Result<Json<PaymentResponse>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    let mut order = state
        .orders
        .get(&id)
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or_else(|| {
            ServiceError::NotFound(state.i18n.t("error.order.not_found", &[("id", &id)]))
        })?;
    if order.buyer.resource_id() != uid {
        return Err(ServiceError::NotFound(
            state.i18n.t("error.order.not_found", &[("id", &id)]),
        ));
    }
    if order.status != "pending_payment" {
        return Err(ServiceError::Validation(
            state.i18n.t("error.order.not_pending", &[]),
        ));
    }
    let result = state.payment.create_payment(&id, order.total_amount);
    if result.success {
        order.status = "paid".into();
        order.paid_at = Some(chrono::Utc::now().to_rfc3339());
        let _ = state.orders.save(order);
    }
    Ok(Json(PaymentResponse {
        success: result.success,
        message: result.message,
    }))
}

// ── E-02-14: Create review ──

pub async fn create_review(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(product_id): Path<String>,
    Json(req): Json<CreateReviewRequest>,
) -> Result<Json<AppReview>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    if req.rating < 1 || req.rating > 5 {
        return Err(ServiceError::Validation(
            state.i18n.t("error.review.invalid_rating", &[]),
        ));
    }

    let purchased_orders: Vec<Order> = state
        .orders
        .list()
        .unwrap_or_default()
        .into_iter()
        .filter(|o| {
            o.buyer.resource_id() == uid
                && (o.status == "paid" || o.status == "shipped" || o.status == "completed")
        })
        .collect();
    let purchased_order = purchased_orders.iter().find(|o| {
        state
            .order_items
            .list()
            .unwrap_or_default()
            .iter()
            .any(|oi| {
                oi.order.resource_id() == o.id.as_str() && oi.product.resource_id() == product_id
            })
    });
    let order = purchased_order
        .ok_or_else(|| ServiceError::Validation(state.i18n.t("error.order.not_purchased", &[])))?;

    let existing = state
        .reviews
        .list()
        .unwrap_or_default()
        .iter()
        .any(|r| r.user.resource_id() == uid && r.product.resource_id() == product_id);
    if existing {
        return Err(ServiceError::Validation(
            state.i18n.t("error.order.already_reviewed", &[]),
        ));
    }

    let user = state.users.get(&uid).ok().flatten();
    let review = state
        .reviews
        .save_new(Review {
            id: Id::default(),
            user: Name::new(format!("shop/users/{}", uid)),
            product: Name::new(format!("shop/products/{}", product_id)),
            order: Name::new(format!("shop/orders/{}", order.id)),
            rating: req.rating,
            content: req.content.clone(),
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

    if let Ok(Some(mut p)) = state.products.get(&product_id) {
        let all_reviews: Vec<Review> = state
            .reviews
            .list()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.product.resource_id() == product_id)
            .collect();
        p.review_count = all_reviews.len() as u32;
        if !all_reviews.is_empty() {
            p.rating = (all_reviews.iter().map(|r| r.rating as u64).sum::<u64>()
                / all_reviews.len() as u64) as u32;
        }
        let _ = state.products.save(p);
    }

    Ok(Json(AppReview {
        id: review.id.to_string(),
        user_id: uid.clone(),
        username: user.map(|u| u.username.clone()).unwrap_or_default(),
        rating: review.rating,
        content: review.content.clone(),
        created_at: review.created_at.to_string(),
    }))
}

// ── E-02-15: Product reviews ──

pub async fn product_reviews(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(product_id): Path<String>,
    Json(params): Json<PaginationParams>,
) -> Result<Json<ReviewListResponse>, ServiceError> {
    let _uid = current_user(&headers, &state)?;
    let all_reviews = state.reviews.list().unwrap_or_default();
    let mut reviews: Vec<&Review> = all_reviews
        .iter()
        .filter(|r| r.product.resource_id() == product_id)
        .collect();
    reviews.sort_by(|a, b| b.created_at.as_str().cmp(a.created_at.as_str()));
    let total = reviews.len();
    let offset = params.offset.min(total);
    let end = (offset + params.limit).min(total);
    let items: Vec<AppReview> = reviews[offset..end]
        .iter()
        .map(|r| {
            let user = state.users.get(r.user.resource_id()).ok().flatten();
            AppReview {
                id: r.id.to_string(),
                user_id: r.user.resource_id().to_string(),
                username: user.map(|u| u.username.clone()).unwrap_or_default(),
                rating: r.rating,
                content: r.content.clone(),
                created_at: r.created_at.to_string(),
            }
        })
        .collect();
    Ok(Json(ReviewListResponse {
        items,
        has_more: end < total,
    }))
}

// ── E-02-16..19: Address CRUD ──

pub async fn address_list(
    headers: HeaderMap,
    State(state): State<FacetState>,
) -> Result<Json<Vec<AppAddress>>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    let addrs: Vec<AppAddress> = state
        .addresses
        .list()
        .unwrap_or_default()
        .iter()
        .filter(|a| a.user.resource_id() == uid)
        .map(to_app_address)
        .collect();
    Ok(Json(addrs))
}

pub async fn create_address(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Json(req): Json<CreateAddressRequest>,
) -> Result<Json<AppAddress>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    if req.is_default {
        let existing = state.addresses.list().unwrap_or_default();
        for mut a in existing
            .into_iter()
            .filter(|a| a.user.resource_id() == uid && a.is_default)
        {
            a.is_default = false;
            let _ = state.addresses.save(a);
        }
    }
    let addr = state
        .addresses
        .save_new(Address {
            id: Id::default(),
            user: Name::new(format!("shop/users/{}", uid)),
            recipient_name: req.recipient_name,
            phone: req.phone,
            province: req.province,
            city: req.city,
            district: req.district,
            detail: req.detail,
            is_default: req.is_default,
            display_name: None,
            description: None,
            metadata: None,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        })
        .map_err(|e| ServiceError::Internal(e.to_string()))?;
    Ok(Json(to_app_address(&addr)))
}

pub async fn update_address(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(id): Path<String>,
    Json(req): Json<CreateAddressRequest>,
) -> Result<Json<AppAddress>, ServiceError> {
    let uid = current_user(&headers, &state)?;
    let mut addr = state
        .addresses
        .get(&id)
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .ok_or_else(|| {
            ServiceError::NotFound(state.i18n.t("error.address.not_found", &[("id", &id)]))
        })?;
    if addr.user.resource_id() != uid {
        return Err(ServiceError::NotFound(
            state.i18n.t("error.address.not_found", &[("id", &id)]),
        ));
    }
    if req.is_default && !addr.is_default {
        let existing = state.addresses.list().unwrap_or_default();
        for mut a in existing
            .into_iter()
            .filter(|a| a.user.resource_id() == uid && a.is_default)
        {
            a.is_default = false;
            let _ = state.addresses.save(a);
        }
    }
    addr.recipient_name = req.recipient_name;
    addr.phone = req.phone;
    addr.province = req.province;
    addr.city = req.city;
    addr.district = req.district;
    addr.detail = req.detail;
    addr.is_default = req.is_default;
    state
        .addresses
        .save(addr.clone())
        .map_err(|e| ServiceError::Internal(e.to_string()))?;
    Ok(Json(to_app_address(&addr)))
}

pub async fn delete_address(
    headers: HeaderMap,
    State(state): State<FacetState>,
    Path(id): Path<String>,
) -> Result<(), ServiceError> {
    let uid = current_user(&headers, &state)?;
    if let Ok(Some(addr)) = state.addresses.get(&id) {
        if addr.user.resource_id() != uid {
            return Err(ServiceError::NotFound(
                state.i18n.t("error.address.not_found", &[("id", &id)]),
            ));
        }
    }
    state
        .addresses
        .delete(&id)
        .map_err(|e| ServiceError::Internal(e.to_string()))?;
    Ok(())
}

// ── Handler completeness checks ──
openerp_macro::impl_handler!(app::Login);
openerp_macro::impl_handler!(app::CategoryList);
openerp_macro::impl_handler!(app::CategoryProducts);
openerp_macro::impl_handler!(app::ProductDetail);
openerp_macro::impl_handler!(app::ProductSearch);
openerp_macro::impl_handler!(app::ShopPage);
openerp_macro::impl_handler!(app::FetchCart);
openerp_macro::impl_handler!(app::AddToCart);
openerp_macro::impl_handler!(app::UpdateCart);
openerp_macro::impl_handler!(app::CreateOrder);
openerp_macro::impl_handler!(app::OrderList);
openerp_macro::impl_handler!(app::OrderDetail);
openerp_macro::impl_handler!(app::PayOrder);
openerp_macro::impl_handler!(app::CreateReview);
openerp_macro::impl_handler!(app::ProductReviews);
openerp_macro::impl_handler!(app::AddressList);
openerp_macro::impl_handler!(app::CreateAddress);
openerp_macro::impl_handler!(app::UpdateAddress);
openerp_macro::impl_handler!(app::DeleteAddress);

pub fn facet_router(state: FacetState) -> axum::Router {
    app::__assert_handlers::<app::__Handlers>();
    use axum::routing::{get, post, put};
    axum::Router::new()
        .route("/auth/login", post(login))
        .route("/categories", get(category_list))
        .route("/categories/{id}/products", post(category_products))
        .route("/products/{id}", get(product_detail))
        .route("/search", post(product_search))
        .route("/shops/{id}", post(shop_page))
        .route("/cart", post(get_cart))
        .route("/cart/add", post(add_to_cart))
        .route("/cart/{id}", put(update_cart))
        .route("/orders", post(create_order))
        .route("/orders/list", post(order_list))
        .route("/orders/{id}", get(order_detail))
        .route("/orders/{id}/pay", post(pay_order))
        .route("/products/{id}/review", post(create_review))
        .route("/products/{id}/reviews", post(product_reviews))
        .route("/addresses", get(address_list).post(create_address))
        .route(
            "/addresses/{id}",
            put(update_address).delete(delete_address),
        )
        .with_state(state)
}
