//! HTTP route registration and the aggregated `OpenAPI` document.

pub mod ap;
pub mod ar;
pub mod calculate;
pub mod health;
pub mod journal;
pub mod ledger;
pub mod narrative;
pub mod simulate;
pub mod treasury;

use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// The aggregated `OpenAPI` document: every path and schema across every
/// implemented route, served as JSON at `/api-docs/openapi.json` and
/// rendered interactively at `/swagger-ui/`.
#[derive(OpenApi)]
#[openapi(
    paths(
        health::healthz,
        calculate::handle_calculate,
        simulate::handle_simulate,
        narrative::generate,
        ledger::register_account,
        ledger::list_accounts,
        ledger::get_account,
        ledger::get_account_balance,
        ledger::trial_balance,
        journal::post_entry,
        journal::list_entries,
        ap::create_supplier,
        ap::list_suppliers,
        ap::create_invoice,
        ap::list_invoices,
        ap::aging,
        ap::propose,
        ar::create_customer,
        ar::list_customers,
        ar::create_invoice,
        ar::list_invoices,
        ar::allocate,
        treasury::add_cashflow_item,
        treasury::projection,
        treasury::shortfall,
        treasury::convert,
        treasury::hedge_effectiveness_handler,
    ),
    tags(
        (name = "narrative", description = "CFO-style narrative memo generation"),
        (name = "ledger", description = "Chart of accounts and balances"),
        (name = "journal", description = "Posting and listing journal entries"),
        (name = "ap", description = "Accounts payable: suppliers, invoices, aging, payment proposals"),
        (name = "ar", description = "Accounts receivable: customers, invoices, receipt allocation"),
        (name = "treasury", description = "Cash forecasting, FX conversion, hedge effectiveness"),
    )
)]
pub struct ApiDoc;

/// Registers every implemented route (including Swagger UI and the raw
/// `OpenAPI` document) onto `cfg`.
///
/// Does **not** create its own [`crate::state::AppState`]: `App::new().configure(...)`'s
/// factory closure runs once per `HttpServer` worker thread, so state
/// created here would be isolated per worker rather than shared. Callers
/// (`main.rs`, and any test that exercises the ERP routes) must
/// `.app_data(web::Data::new(AppState::new()))` themselves, once, before
/// calling this.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(health::healthz));
    cfg.service(
        web::scope("/api/v1")
            .route(
                "/calculate/{formula}",
                web::post().to(calculate::handle_calculate),
            )
            .route("/simulate", web::post().to(simulate::handle_simulate))
            .route("/narrative", web::post().to(narrative::generate))
            .service(
                web::scope("/ledger")
                    .route("/accounts", web::post().to(ledger::register_account))
                    .route("/accounts", web::get().to(ledger::list_accounts))
                    .route("/accounts/{code}", web::get().to(ledger::get_account))
                    .route(
                        "/accounts/{code}/balance",
                        web::get().to(ledger::get_account_balance),
                    )
                    .route("/trial-balance", web::get().to(ledger::trial_balance)),
            )
            .service(
                web::scope("/journal")
                    .route("/entries", web::post().to(journal::post_entry))
                    .route("/entries", web::get().to(journal::list_entries)),
            )
            .service(
                web::scope("/ap")
                    .route("/suppliers", web::post().to(ap::create_supplier))
                    .route("/suppliers", web::get().to(ap::list_suppliers))
                    .route("/invoices", web::post().to(ap::create_invoice))
                    .route("/invoices", web::get().to(ap::list_invoices))
                    .route("/aging", web::get().to(ap::aging))
                    .route("/payments/propose", web::post().to(ap::propose)),
            )
            .service(
                web::scope("/ar")
                    .route("/customers", web::post().to(ar::create_customer))
                    .route("/customers", web::get().to(ar::list_customers))
                    .route("/invoices", web::post().to(ar::create_invoice))
                    .route("/invoices", web::get().to(ar::list_invoices))
                    .route("/receipts/allocate", web::post().to(ar::allocate)),
            )
            .service(
                web::scope("/treasury")
                    .route(
                        "/cashflow/items",
                        web::post().to(treasury::add_cashflow_item),
                    )
                    .route("/cashflow/projection", web::get().to(treasury::projection))
                    .route("/cashflow/shortfall", web::get().to(treasury::shortfall))
                    .route("/fx/convert", web::post().to(treasury::convert))
                    .route(
                        "/hedge/effectiveness",
                        web::post().to(treasury::hedge_effectiveness_handler),
                    ),
            ),
    );
    cfg.route("/ws/simulate", web::get().to(simulate::ws_simulate));
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()),
    );
}
