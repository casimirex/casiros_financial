import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { ScrollText, TriangleAlert } from "lucide-react";
import { Shell } from "@/components/layout/shell";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { ApiError } from "@/api/client";
import { narrativeApi } from "@/api/narrative";
import type { NarrativeInputs } from "@/api/types";

const METRIC_FIELDS: { key: keyof Omit<NarrativeInputs, "company">; label: string; placeholder: string }[] = [
  { key: "roe", label: "Return on Equity", placeholder: "0.15" },
  { key: "roa", label: "Return on Assets", placeholder: "0.08" },
  { key: "debt_to_equity", label: "Debt to Equity", placeholder: "0.8" },
  { key: "current_ratio", label: "Current Ratio", placeholder: "2.0" },
  { key: "quick_ratio", label: "Quick Ratio", placeholder: "1.2" },
  { key: "profit_margin", label: "Profit Margin", placeholder: "0.10" },
  { key: "net_income", label: "Net Income", placeholder: "50000" },
  { key: "interest_coverage", label: "Interest Coverage", placeholder: "4.0" },
  { key: "asset_turnover", label: "Asset Turnover", placeholder: "1.0" },
];

function renderMemo(markdown: string) {
  return markdown
    .trim()
    .split("\n\n")
    .filter(Boolean)
    .map((block, i) => {
      if (block.startsWith("## ")) {
        return (
          <h2 key={i} className="text-gradient-signal text-2xl font-bold">
            {block.replace("## ", "")}
          </h2>
        );
      }
      const boldMatch = block.match(/^\*\*(.+?)\*\* of (.+?) is (.+)\.$/);
      if (boldMatch) {
        const [, label, value, interpretation] = boldMatch;
        return (
          <p key={i} className="leading-relaxed text-void-200">
            <span className="font-semibold text-void-50">{label}</span> of{" "}
            <span className="font-mono text-signal-400">{value}</span> is {interpretation}.
          </p>
        );
      }
      return (
        <p key={i} className="leading-relaxed text-void-200">
          {block}
        </p>
      );
    });
}

export function NarrativePage() {
  const [company, setCompany] = useState("Acme Corp");
  const [metrics, setMetrics] = useState<Record<string, string>>({
    roe: "0.15",
    debt_to_equity: "0.8",
    current_ratio: "2.0",
  });

  const mutation = useMutation({
    mutationFn: () => {
      const inputs: NarrativeInputs = { company };
      for (const { key } of METRIC_FIELDS) {
        const raw = metrics[key];
        if (raw) inputs[key] = raw;
      }
      return narrativeApi.generate(inputs);
    },
  });

  return (
    <Shell title="Narrative" subtitle="Generate a CFO-style memo from whichever metrics you supply.">
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Metrics</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-1">
              <Label htmlFor="company">Company</Label>
              <Input id="company" value={company} onChange={(e) => setCompany(e.target.value)} />
            </div>
            <div className="grid grid-cols-2 gap-3">
              {METRIC_FIELDS.map(({ key, label, placeholder }) => (
                <div key={key} className="space-y-1">
                  <Label htmlFor={key}>{label}</Label>
                  <Input
                    id={key}
                    placeholder={placeholder}
                    value={metrics[key] ?? ""}
                    onChange={(e) => setMetrics((m) => ({ ...m, [key]: e.target.value }))}
                  />
                </div>
              ))}
            </div>
            <p className="text-xs text-void-500">
              Leave any metric blank to omit it from the memo — every field is optional except
              company.
            </p>
            <Button className="w-full" onClick={() => mutation.mutate()} disabled={mutation.isPending}>
              <ScrollText className="h-4 w-4" />
              {mutation.isPending ? "Generating..." : "Generate Memo"}
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="min-h-[24rem] space-y-4 py-8">
            {mutation.isIdle && (
              <p className="text-center text-sm text-void-500">
                Fill in the metrics you have and generate the memo.
              </p>
            )}
            {mutation.isError && (
              <div className="flex flex-col items-center gap-2 py-16 text-unfavorable-500">
                <TriangleAlert className="h-6 w-6" />
                <p className="text-sm">
                  {mutation.error instanceof ApiError
                    ? mutation.error.message
                    : "Something went wrong."}
                </p>
              </div>
            )}
            {mutation.isSuccess && <div className="space-y-4">{renderMemo(mutation.data.memo)}</div>}
          </CardContent>
        </Card>
      </div>
    </Shell>
  );
}
