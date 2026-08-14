-- Journal entries and their debit/credit lines. causal_parent deliberately
-- has no foreign key: casiros_erp::ledger::journal::JournalEntry::new never
-- validated it against existing entries, and the frontend's causality
-- lineage view already documents (and handles) a dangling reference — an
-- FK here would turn a request that succeeds today into a new error.
CREATE TABLE journal_entries (
    id            UUID PRIMARY KEY,
    seq           BIGSERIAL,
    date          DATE NOT NULL,
    description   TEXT NOT NULL,
    causal_parent UUID,
    source_kind   TEXT NOT NULL
        CHECK (source_kind IN ('ManualEntry', 'Invoice', 'Payment', 'Receipt', 'Accrual', 'Consolidation')),
    source_id     UUID,
    period_year   INT NOT NULL,
    period_month  SMALLINT NOT NULL CHECK (period_month BETWEEN 1 AND 12)
);

-- seq preserves posting order for display purposes; never serialized to JSON.
CREATE INDEX idx_journal_entries_seq ON journal_entries(seq);
CREATE INDEX idx_journal_entries_causal_parent ON journal_entries(causal_parent);
CREATE INDEX idx_journal_entries_period ON journal_entries(period_year, period_month);

CREATE TABLE journal_lines (
    id             BIGSERIAL PRIMARY KEY,
    entry_id       UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    line_ordinal   INT NOT NULL,
    account        BIGINT NOT NULL REFERENCES accounts(code),
    debit          NUMERIC NOT NULL DEFAULT 0 CHECK (debit >= 0),
    credit         NUMERIC NOT NULL DEFAULT 0 CHECK (credit >= 0),
    causal_formula TEXT,
    UNIQUE (entry_id, line_ordinal)
);

-- The balance-aggregation query (SUM(debit - credit) GROUP BY account) is
-- the whole reason leaf balances no longer need incremental maintenance.
CREATE INDEX idx_journal_lines_account ON journal_lines(account);
