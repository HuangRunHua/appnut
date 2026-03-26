//! Payment abstraction — trait + mock implementation.

/// Result of a payment attempt.
pub struct PaymentResult {
    pub success: bool,
    pub message: String,
    pub payment_id: String,
}

/// Abstract payment provider — swap in real gateways later.
pub trait PaymentProvider: Send + Sync + 'static {
    fn create_payment(&self, order_id: &str, amount: u64) -> PaymentResult;
    fn query_payment(&self, payment_id: &str) -> bool;
}

/// Mock payment — always succeeds, for development and testing.
pub struct MockPaymentProvider;

impl PaymentProvider for MockPaymentProvider {
    fn create_payment(&self, order_id: &str, amount: u64) -> PaymentResult {
        PaymentResult {
            success: true,
            message: format!("Mock payment of {} cents for order {}", amount, order_id),
            payment_id: format!("mock_pay_{}", order_id),
        }
    }

    fn query_payment(&self, _payment_id: &str) -> bool {
        true
    }
}
