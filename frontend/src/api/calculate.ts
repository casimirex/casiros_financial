import { api } from "./client";
import type { CalculateRequest, CalculateResponse } from "./types";

export const calculateApi = {
  evaluate: (formula: string, params: Record<string, string>) =>
    api.post<CalculateResponse>(`/api/v1/calculate/${formula}`, {
      params,
    } satisfies CalculateRequest),
};
