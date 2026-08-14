import { api } from "./client";
import type { Account, AccountBalance, JournalEntry, PostJournalEntryRequest } from "./types";

export const ledgerApi = {
  listAccounts: () => api.get<Account[]>("/api/v1/ledger/accounts"),
  registerAccount: (account: Account) => api.post<Account>("/api/v1/ledger/accounts", account),
  getAccount: (code: number) => api.get<Account>(`/api/v1/ledger/accounts/${code}`),
  getBalance: (code: number) =>
    api.get<AccountBalance>(`/api/v1/ledger/accounts/${code}/balance`),
  trialBalance: () => api.get<AccountBalance[]>("/api/v1/ledger/trial-balance"),
  listEntries: () => api.get<JournalEntry[]>("/api/v1/journal/entries"),
  postEntry: (entry: PostJournalEntryRequest) =>
    api.post<JournalEntry>("/api/v1/journal/entries", entry),
};
