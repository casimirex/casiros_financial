-- Suppliers and their AP invoices.
--
-- No UNIQUE(supplier_id, invoice_number): the in-memory HashMap<ApInvoiceId,
-- _> never enforced that either, only id uniqueness — don't tighten
-- behavior nobody asked to tighten.
--
-- No FOREIGN KEY on payable_account or supplier_id either, for the same
-- reason: routes/ap.rs's create_supplier and create_invoice handlers never
-- validated `payable_account` against the chart of accounts or `supplier`
-- against the suppliers map before storing — both are declared "infallible"
-- (create_supplier) or fallible only on a non-positive amount
-- (create_invoice) in their own doc comments. Adding referential integrity
-- here would silently turn requests that succeed today into new 404s.
CREATE TABLE suppliers (
    id               UUID PRIMARY KEY,
    name             TEXT NOT NULL,
    net_days         INT NOT NULL,
    discount_percent NUMERIC,
    discount_days    INT,
    payable_account  BIGINT NOT NULL
);

CREATE TABLE ap_invoices (
    id               UUID PRIMARY KEY,
    supplier_id      UUID NOT NULL,
    invoice_number   TEXT NOT NULL,
    invoice_date     DATE NOT NULL,
    amount           NUMERIC NOT NULL CHECK (amount > 0),
    net_days         INT NOT NULL,
    discount_percent NUMERIC,
    discount_days    INT,
    amount_paid      NUMERIC NOT NULL DEFAULT 0,
    status           TEXT NOT NULL CHECK (status IN ('Open', 'PartiallyPaid', 'Paid'))
);

CREATE INDEX idx_ap_invoices_supplier ON ap_invoices(supplier_id);
