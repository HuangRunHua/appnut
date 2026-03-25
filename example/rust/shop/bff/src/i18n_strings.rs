//! ShopFlux i18n translations — en + zh-CN.

use std::collections::HashMap;
use std::sync::Arc;

use openerp_flux::{I18nHandler, I18nStore, QueryParams};

pub fn register_all(i18n: &I18nStore) {
    i18n.handle("ui/#", Arc::new(UiStrings::new()));
    i18n.handle("error/#", Arc::new(ErrorStrings::new()));
}

const EN: usize = 0;
const ZH: usize = 1;

fn locale_index(locale: &str) -> usize {
    match locale {
        "zh-CN" | "zh" => ZH,
        _ => EN,
    }
}

struct UiStrings {
    data: HashMap<&'static str, [&'static str; 2]>,
}

impl UiStrings {
    fn new() -> Self {
        let mut m = HashMap::new();
        m.insert("ui/login/title", ["Welcome to ShopFlux", "欢迎来到 ShopFlux"]);
        m.insert("ui/login/button", ["Sign In", "登录"]);
        m.insert("ui/tab/home", ["Home", "首页"]);
        m.insert("ui/tab/categories", ["Categories", "分类"]);
        m.insert("ui/tab/cart", ["Cart", "购物车"]);
        m.insert("ui/tab/orders", ["Orders", "订单"]);
        m.insert("ui/tab/me", ["Me", "我"]);
        m.insert("ui/search/placeholder", ["Search products...", "搜索商品..."]);
        m.insert("ui/cart/empty", ["Your cart is empty", "购物车为空"]);
        m.insert("ui/cart/checkout", ["Checkout", "去结算"]);
        m.insert("ui/order/pay", ["Pay Now", "立即支付"]);
        m.insert("ui/order/status/pending_payment", ["Pending Payment", "待付款"]);
        m.insert("ui/order/status/paid", ["Paid", "已付款"]);
        m.insert("ui/order/status/shipped", ["Shipped", "已发货"]);
        m.insert("ui/order/status/completed", ["Completed", "已完成"]);
        m.insert("ui/order/status/cancelled", ["Cancelled", "已取消"]);
        m.insert("ui/review/write", ["Write a Review", "写评价"]);
        m.insert("ui/address/add", ["Add Address", "添加地址"]);
        m.insert("ui/common/loading", ["Loading...", "加载中..."]);
        Self { data: m }
    }
}

impl I18nHandler for UiStrings {
    fn translate(&self, path: &str, _query: &QueryParams, locale: &str) -> String {
        let idx = locale_index(locale);
        self.data
            .get(path)
            .map(|t| t[idx].to_string())
            .unwrap_or_else(|| path.to_string())
    }
}

struct ErrorStrings {
    data: HashMap<&'static str, [&'static str; 2]>,
}

impl ErrorStrings {
    fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(
            "error/auth/missing_token",
            ["Missing Authorization header", "缺少认证信息"],
        );
        m.insert(
            "error/auth/invalid_token",
            ["Invalid or expired token", "无效或过期的令牌"],
        );
        m.insert("error/auth/user_not_found", ["User not found", "用户不存在"]);
        m.insert("error/product/not_found", ["Product not found", "商品不存在"]);
        m.insert("error/order/not_found", ["Order not found", "订单不存在"]);
        m.insert(
            "error/stock/insufficient",
            ["Insufficient stock", "库存不足"],
        );
        Self { data: m }
    }
}

impl I18nHandler for ErrorStrings {
    fn translate(&self, path: &str, _query: &QueryParams, locale: &str) -> String {
        let idx = locale_index(locale);
        self.data
            .get(path)
            .map(|t| t[idx].to_string())
            .unwrap_or_else(|| path.to_string())
    }
}
