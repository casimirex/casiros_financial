import { api } from "./client";
import type { ArInvoice, Customer, PaymentTerms, ReceiptAllocation, RecognitionMethod } from "./types";

export interface CreateCustomerRequest {
  name: string;
  credit_limit: string;
  payment_terms: PaymentTerms;
  receivable_account: number;
}

export interface CreateArInvoiceRequest {
  customer: string;
  invoice_number: string;
  invoice_date: string;
  amount: string;
  terms: PaymentTerms;
  recognition_method: RecognitionMethod;
}

export interface AllocateReceiptRequest {
  customer: string;
  amount: string;
  date: string;
}

export const arApi = {
  listCustomers: () => api.get<Customer[]>("/api/v1/ar/customers"),
  createCustomer: (request: CreateCustomerRequest) =>
    api.post<Customer>("/api/v1/ar/customers", request),
  listInvoices: () => api.get<ArInvoice[]>("/api/v1/ar/invoices"),
  createInvoice: (request: CreateArInvoiceRequest) =>
    api.post<ArInvoice>("/api/v1/ar/invoices", request),
  allocateReceipt: (request: AllocateReceiptRequest) =>
    api.post<ReceiptAllocation[]>("/api/v1/ar/receipts/allocate", request),
};
