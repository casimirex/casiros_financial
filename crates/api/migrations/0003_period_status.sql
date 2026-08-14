-- Existence of a row = Closed; absence = Open. Matches the in-memory
-- Ledger's `period_status: HashMap<FiscalPeriod, PeriodStatus>`, which only
-- ever stores an entry when a period is explicitly closed.
CREATE TABLE closed_periods (
    year  INT NOT NULL,
    month SMALLINT NOT NULL CHECK (month BETWEEN 1 AND 12),
    PRIMARY KEY (year, month)
);
