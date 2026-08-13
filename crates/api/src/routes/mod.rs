//! HTTP route registration.
//!
//! Implemented: `/healthz`, `/api/v1/calculate/{formula}`, `/api/v1/simulate`,
//! `/ws/simulate`. Not yet implemented (follow-up work): `/api/v1/ledger/*`,
//! `/api/v1/journal/*`, `/api/v1/ap/*`, `/api/v1/ar/*`, `/api/v1/treasury/*`
//! — each is a substantial CRUD surface over `casiros-erp` and deserves its
//! own pass, the same way the ERP crate's subsystems were each built
//! separately in Phase 4.

pub mod calculate;
pub mod health;
pub mod simulate;

use actix_web::web;

/// Registers every implemented route onto `cfg`.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(health::healthz));
    cfg.service(
        web::scope("/api/v1")
            .route(
                "/calculate/{formula}",
                web::post().to(calculate::handle_calculate),
            )
            .route("/simulate", web::post().to(simulate::handle_simulate)),
    );
    cfg.route("/ws/simulate", web::get().to(simulate::ws_simulate));
}
