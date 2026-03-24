//! Server-side module registration interface.
//!
//! [`ServerModule`] extends [`FluxModule`] with server-specific
//! configuration: facet routes, admin routes, and seed data.
//!
//! ```ignore
//! struct TwitterModule { /* ... */ }
//!
//! impl FluxModule for TwitterModule {
//!     fn name(&self) -> &str { "twitter" }
//!     fn register_handlers(&self, flux: &Flux) { /* ... */ }
//! }
//!
//! impl ServerModule for TwitterModule {
//!     fn facet_router(&self, ctx: &ServerContext) -> axum::Router {
//!         facet_handlers::facet_router(ctx)
//!     }
//!     fn admin_router(&self, ctx: &ServerContext) -> axum::Router {
//!         server::admin_router(ctx.kv.clone(), ctx.auth.clone())
//!     }
//! }
//! ```

use std::sync::Arc;

pub use openerp_flux::FluxModule;

/// Server-side context provided to [`ServerModule`] methods during setup.
///
/// Contains the storage backends and server URL created by the
/// framework during embedded server initialization.
pub struct ServerContext {
    /// Key-value store (Redb) for persistent data.
    pub kv: Arc<dyn openerp_kv::KVStore>,
    /// Blob store (file system) for binary assets.
    pub blobs: Arc<dyn openerp_blob::BlobStore>,
    /// Authenticator for protecting admin/facet endpoints.
    pub auth: Arc<dyn openerp_core::Authenticator>,
    /// Base URL of the embedded server (e.g. "http://192.168.1.100:3000").
    pub server_url: String,
}

/// Server-side extension of [`FluxModule`].
///
/// Provides the HTTP routes and seed data that the embedded server
/// needs to serve your application's REST API and admin dashboard.
pub trait ServerModule: FluxModule {
    /// Build the Facet (REST API) router for this module.
    ///
    /// The returned router is mounted at `/app/{name}/*`.
    fn facet_router(&self, ctx: &ServerContext) -> axum::Router;

    /// Build the Admin (CRUD dashboard) router for this module.
    ///
    /// The returned router is mounted at `/admin/{name}/*`.
    fn admin_router(&self, ctx: &ServerContext) -> axum::Router;

    /// Load seed data for development mode (optional).
    ///
    /// Called once during server initialization, before any
    /// client connects. Override to populate demo data.
    fn seed_data(&self, _ctx: &ServerContext) {}
}
