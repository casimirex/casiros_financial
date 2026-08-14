-- Suppliers and their AP invoices. No UNIQUE(supplier_id, invoice_number):
-- the in-memory HashMap<ApInvoiceId, _> never enforced that either, only id
-- uniqueness — don't tighten behavior nobody asked to tighten.
CREATE TABLE suppliers (
    id               UUID PRIMARY KEY,
    name             TEXT NOT NULL,
    net_days         INT NOT NULL,
    discount_percent NUMERIC,
    discount_days    INT,
    payable_account  BIGINT NOT NULL REFERENCES accounts(code)
);

CREATE TABLE ap_invoices (
    id               UUID PRIMARY KEY,
    supplier_id      UUID NOT NULL REFERENCES suppliers(id),
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
