import { api } from "./client";

export interface HealthResponse {
  status: string;
}

export const healthApi = {
  check: () => api.get<HealthResponse>("/healthz"),
};
