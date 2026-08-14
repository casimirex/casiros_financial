-- Chart of accounts. code is the same u32 AccountCode the domain layer uses;
-- BIGINT (not INT) so the full u32 range is representable without a cast.
CREATE TABLE accounts (
    code         BIGINT PRIMARY KEY,
    name         TEXT NOT NULL,
    account_type TEXT NOT NULL
        CHECK (account_type IN ('Asset', 'Liability', 'Equity', 'Revenue', 'Expense')),
    parent_code  BIGINT REFERENCES accounts(code)
);

CREATE INDEX idx_accounts_parent ON accounts(parent_code);
