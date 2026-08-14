import { api } from "./client";
import type { SimulateRequest, SimulateResponse } from "./types";

export const simulateApi = {
  run: (request: SimulateRequest) => api.post<SimulateResponse>("/api/v1/simulate", request),
};
