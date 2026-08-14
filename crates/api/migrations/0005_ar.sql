-- Customers and their AR invoices. recognition_kind/date/start/end together
-- encode casiros_erp::ar::invoice::RecognitionMethod's two variants; the
-- CHECK enforces exactly the fields that variant carries are populated.
--
-- No FOREIGN KEY on receivable_account or customer_id: routes/ar.rs's
-- create_customer and create_invoice handlers never validated those
-- references against the chart of accounts or the customers list either
-- (create_customer's own doc comment calls it "infallible") — same
-- treatment as 0004_ap.sql's suppliers/ap_invoices for the same reason.
CREATE TABLE customers (
    id                 UUID PRIMARY KEY,
    name               TEXT NOT NULL,
    credit_limit       NUMERIC NOT NULL,
    net_days           INT NOT NULL,
    discount_percent   NUMERIC,
    discount_days      INT,
    receivable_account BIGINT NOT NULL
);

CREATE TABLE ar_invoices (
    id                 UUID PRIMARY KEY,
    customer_id        UUID NOT NULL,
    invoice_number     TEXT NOT NULL,
    invoice_date       DATE NOT NULL,
    amount             NUMERIC NOT NULL CHECK (amount > 0),
    net_days           INT NOT NULL,
    discount_percent   NUMERIC,
    discount_days      INT,
    recognition_kind   TEXT NOT NULL CHECK (recognition_kind IN ('PointInTime', 'RatablyOverTime')),
    recognition_date   DATE,
    recognition_start  DATE,
    recognition_end    DATE,
    amount_received    NUMERIC NOT NULL DEFAULT 0,
    status             TEXT NOT NULL CHECK (status IN ('Open', 'PartiallyCollected', 'Collected')),
    CHECK (
        (recognition_kind = 'PointInTime'
            AND recognition_date IS NOT NULL
            AND recognition_start IS NULL AND recognition_end IS NULL)
        OR
        (recognition_kind = 'RatablyOverTime'
            AND recognition_start IS NOT NULL AND recognition_end IS NOT NULL
            AND recognition_date IS NULL)
    )
);

CREATE INDEX idx_ar_invoices_customer ON ar_invoices(customer_id);
