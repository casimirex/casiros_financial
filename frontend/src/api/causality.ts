import { api } from "./client";
import type { DependencyGraphResponse, FormulaMetadata } from "./types";

export const causalityApi = {
  listFormulas: () => api.get<FormulaMetadata[]>("/api/v1/causality/formulas"),
  formulaGraph: (snakeCaseName: string) =>
    api.get<DependencyGraphResponse>(`/api/v1/causality/formulas/${snakeCaseName}`),
};
