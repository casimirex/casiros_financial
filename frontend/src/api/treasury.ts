import { api } from "./client";
import type { CashFlowItem, ExchangeRate, FxExposure } from "./types";

export interface ProjectionResponse {
  balance: string;
}

export interface ShortfallResponse {
  shortfall_date: string | null;
}

export interface ConvertRequest {
  exposure: FxExposure;
  rate: ExchangeRate;
}

export interface ConvertResponse {
  converted: string;
}

export interface HedgeEffectivenessRequest {
  hedge_gain_loss: string;
  exposure_gain_loss: string;
}

export interface HedgeEffectivenessResponse {
  effectiveness: string;
  highly_effective: boolean;
}

export const treasuryApi = {
  addCashflowItem: (item: CashFlowItem) =>
    api.post<CashFlowItem>("/api/v1/treasury/cashflow/items", item),
  projection: (openingBalance: string, asOf: string) =>
    api.get<ProjectionResponse>(
      `/api/v1/treasury/cashflow/projection?opening_balance=${openingBalance}&as_of=${asOf}`,
    ),
  shortfall: (openingBalance: string) =>
    api.get<ShortfallResponse>(
      `/api/v1/treasury/cashflow/shortfall?opening_balance=${openingBalance}`,
    ),
  convert: (request: ConvertRequest) =>
    api.post<ConvertResponse>("/api/v1/treasury/fx/convert", request),
  hedgeEffectiveness: (request: HedgeEffectivenessRequest) =>
    api.post<HedgeEffectivenessResponse>("/api/v1/treasury/hedge/effectiveness", request),
};
