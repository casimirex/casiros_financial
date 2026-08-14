import { api } from "./client";
import type { DeferredTaxPosition, TaxJurisdiction, TemporaryDifference } from "./types";

export interface CalculateTaxRequest {
  jurisdiction: TaxJurisdiction;
  taxable_income: string;
}

export interface CalculateTaxResponse {
  tax: string;
}

export interface JurisdictionAllocation {
  jurisdiction: TaxJurisdiction;
  taxable_income: string;
}

export interface MultiJurisdictionResponse {
  total_tax: string;
}

export const taxApi = {
  calculate: (request: CalculateTaxRequest) =>
    api.post<CalculateTaxResponse>("/api/v1/tax/calculate", request),
  multiJurisdiction: (allocations: JurisdictionAllocation[]) =>
    api.post<MultiJurisdictionResponse>("/api/v1/tax/multi-jurisdiction", { allocations }),
  deferredPosition: (request: TemporaryDifference) =>
    api.post<DeferredTaxPosition>("/api/v1/tax/deferred-position", request),
};
