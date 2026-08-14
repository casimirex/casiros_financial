import { api } from "./client";
import type { DriverBasedLineItem, VarianceEntry, VarianceResult } from "./types";

export interface SetDriverRequest {
  name: string;
  value: string;
}

export interface DriverResponse {
  value: string;
}

export interface TotalBudgetResponse {
  total: string;
}

export const budgetApi = {
  setDriver: (request: SetDriverRequest) =>
    api.post<SetDriverRequest>("/api/v1/budget/drivers", request),
  getDriver: (name: string) => api.get<DriverResponse>(`/api/v1/budget/drivers/${name}`),
  listLineItems: () => api.get<DriverBasedLineItem[]>("/api/v1/budget/line-items"),
  addLineItem: (item: DriverBasedLineItem) =>
    api.post<DriverBasedLineItem>("/api/v1/budget/line-items", item),
  total: () => api.get<TotalBudgetResponse>("/api/v1/budget/total"),
  variance: (entries: VarianceEntry[]) =>
    api.post<VarianceResult[]>("/api/v1/budget/variance", { entries }),
};
