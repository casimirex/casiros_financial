import { api } from "./client";
import type { AgingReport, ApInvoice, PaymentProposal, PaymentTerms, Supplier } from "./types";

export interface CreateSupplierRequest {
  name: string;
  payment_terms: PaymentTerms;
  payable_account: number;
}

export interface CreateApInvoiceRequest {
  supplier: string;
  invoice_number: string;
  invoice_date: string;
  amount: string;
  terms: PaymentTerms;
}

export interface ProposePaymentsRequest {
  as_of: string;
  available_cash: string;
  current_liabilities: string;
}

export const apApi = {
  listSuppliers: () => api.get<Supplier[]>("/api/v1/ap/suppliers"),
  createSupplier: (request: CreateSupplierRequest) =>
    api.post<Supplier>("/api/v1/ap/suppliers", request),
  listInvoices: () => api.get<ApInvoice[]>("/api/v1/ap/invoices"),
  createInvoice: (request: CreateApInvoiceRequest) =>
    api.post<ApInvoice>("/api/v1/ap/invoices", request),
  aging: (asOf: string) => api.get<AgingReport>(`/api/v1/ap/aging?as_of=${asOf}`),
  propose: (request: ProposePaymentsRequest) =>
    api.post<PaymentProposal[]>("/api/v1/ap/payments/propose", request),
};
