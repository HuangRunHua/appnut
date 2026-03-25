//! Internationalization for ShopFlux.

pub trait Localizer: Send + Sync + 'static {
    fn t(&self, key: &str, args: &[(&str, &str)]) -> String;
}

pub struct DefaultLocalizer;

impl Localizer for DefaultLocalizer {
    fn t(&self, key: &str, args: &[(&str, &str)]) -> String {
        let text = match key {
            "error.auth.missing_token" => "Missing Authorization header",
            "error.auth.invalid_token" => "Invalid or expired token",
            "error.auth.user_not_found" => "User '{username}' not found",
            "error.product.not_found" => "Product '{id}' not found",
            "error.shop.not_found" => "Shop '{id}' not found",
            "error.category.not_found" => "Category '{id}' not found",
            "error.order.not_found" => "Order '{id}' not found",
            "error.order.not_pending" => "Order is not pending payment",
            "error.order.already_reviewed" => "You already reviewed this product",
            "error.order.not_purchased" => "You must purchase this product before reviewing",
            "error.address.not_found" => "Address '{id}' not found",
            "error.cart.empty_selection" => "No items selected for order",
            "error.stock.insufficient" => "Insufficient stock for '{title}'",
            "error.review.invalid_rating" => "Rating must be between 1 and 5",
            "error.internal" => "Internal server error",
            _ => return key.to_string(),
        };
        let mut result = text.to_string();
        for (name, value) in args {
            result = result.replace(&format!("{{{}}}", name), value);
        }
        result
    }
}
