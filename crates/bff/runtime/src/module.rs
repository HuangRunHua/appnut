//! Application module registration interface.
//!
//! Implement [`FluxModule`] to plug your BFF into the Flux framework.
//!
//! ```ignore
//! struct MyBff { /* ... */ }
//!
//! impl FluxModule for MyBff {
//!     fn name(&self) -> &str { "myapp" }
//!
//!     fn register_handlers(&self, flux: &Flux) {
//!         // delegates to macro-generated register()
//!         self.inner.register(flux);
//!     }
//! }
//! ```

use crate::Flux;
use crate::I18nStore;

/// An application module that plugs into the Flux framework.
///
/// Defines the core BFF registration interface: handler registration,
/// i18n strings, and schema definition. These are framework-level
/// concerns that don't depend on the server runtime.
///
/// For server-specific configuration (facet routes, admin routes,
/// seed data), see the FFI layer's `ServerModule` trait.
pub trait FluxModule: Send + Sync {
    /// A unique name for this module (e.g. "twitter", "ecommerce").
    ///
    /// Used as the URL path prefix for facet and admin routes
    /// (e.g. `/app/twitter/*`, `/admin/twitter/*`).
    fn name(&self) -> &str;

    /// Called after the embedded server is ready, with its URL.
    ///
    /// Use this to initialize components that need the server URL
    /// (e.g. HTTP clients for calling facet APIs).
    fn on_server_ready(&self, _server_url: &str) {}

    /// Register all BFF handlers with the Flux engine.
    ///
    /// Called after [`on_server_ready`](Self::on_server_ready).
    /// Typically delegates to the macro-generated `register()` method:
    /// ```ignore
    /// fn register_handlers(&self, flux: &Flux) {
    ///     self.bff.register(flux);
    /// }
    /// ```
    fn register_handlers(&self, flux: &Flux);

    /// Register i18n translation strings (optional).
    ///
    /// Override to add locale-specific UI strings that the client
    /// can query via `flux_i18n_get`.
    fn register_i18n(&self, _store: &I18nStore) {}

    /// Return the schema definition for the admin dashboard (optional).
    ///
    /// The returned JSON is served at `/meta/schema` for the
    /// dashboard to discover models, fields, and relationships.
    fn schema(&self) -> serde_json::Value {
        serde_json::Value::Null
    }
}
