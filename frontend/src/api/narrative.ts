import { api } from "./client";
import type { NarrativeInputs, NarrativeResponse } from "./types";

export const narrativeApi = {
  generate: (inputs: NarrativeInputs) =>
    api.post<NarrativeResponse>("/api/v1/narrative", inputs),
};
