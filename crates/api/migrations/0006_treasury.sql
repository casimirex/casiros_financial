-- Cash flow forecast items. CashFlowItem has no id field in the domain type
-- (see crates/erp/src/treasury/cashflow.rs) — id here is a storage-only
-- surrogate. Read path orders by id to preserve the insertion-order
-- tie-breaking CashForecast::first_shortfall_date's stable sort relies on.
CREATE TABLE cash_flow_items (
    id          BIGSERIAL PRIMARY KEY,
    category    TEXT NOT NULL CHECK (category IN ('Operating', 'Investing', 'Financing')),
    description TEXT NOT NULL,
    amount      NUMERIC NOT NULL,
    date        DATE NOT NULL
);
