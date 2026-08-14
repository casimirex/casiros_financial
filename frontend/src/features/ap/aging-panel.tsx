import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { apApi } from "@/api/ap";
import type { AgingReport } from "@/api/types";
import { formatCurrency, toNumber } from "@/lib/format";
import { cn } from "@/lib/utils";

const BUCKETS: { key: keyof AgingReport; label: string; className: string }[] = [
  { key: "current", label: "Current", className: "bg-favorable-500" },
  { key: "days_1_to_30", label: "1–30 days", className: "bg-signal-500" },
  { key: "days_31_to_60", label: "31–60 days", className: "bg-caution-500" },
  { key: "days_61_to_90", label: "61–90 days", className: "bg-nova-500" },
  { key: "over_90", label: "90+ days", className: "bg-unfavorable-500" },
];

export function AgingPanel() {
  const [asOf, setAsOf] = useState(new Date().toISOString().slice(0, 10));
  const agingQuery = useQuery({
    queryKey: ["ap-aging", asOf],
    queryFn: () => apApi.aging(asOf),
    enabled: Boolean(asOf),
  });

  const report = agingQuery.data;
  const total = report ? BUCKETS.reduce((sum, b) => sum + toNumber(report[b.key]), 0) : 0;

  return (
    <Card>
      <CardHeader className="flex-row items-center justify-between">
        <CardTitle>AP aging report</CardTitle>
        <div className="flex items-center gap-2">
          <Label htmlFor="ap-aging-date" className="text-xs text-void-500">
            As of
          </Label>
          <Input
            id="ap-aging-date"
            type="date"
            className="h-8 w-40"
            value={asOf}
            onChange={(e) => setAsOf(e.target.value)}
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {total === 0 ? (
          <p className="text-sm text-void-500">No outstanding balance as of {asOf}.</p>
        ) : (
          <>
            <div className="flex h-4 w-full overflow-hidden rounded-full bg-void-800">
              {BUCKETS.map((bucket) => {
                const value = report ? toNumber(report[bucket.key]) : 0;
                const pct = total > 0 ? (value / total) * 100 : 0;
                if (pct <= 0) return null;
                return (
                  <div
                    key={bucket.key}
                    className={cn("h-full", bucket.className)}
                    style={{ width: `${pct}%` }}
                    title={`${bucket.label}: ${formatCurrency(value.toFixed(2))}`}
                  />
                );
              })}
            </div>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-5">
              {BUCKETS.map((bucket) => (
                <div key={bucket.key} className="space-y-1">
                  <div className="flex items-center gap-1.5 text-xs text-void-500">
                    <span className={cn("h-2 w-2 rounded-full", bucket.className)} />
                    {bucket.label}
                  </div>
                  <div className="font-mono text-sm text-void-100">
                    {formatCurrency(report ? report[bucket.key] : "0")}
                  </div>
                </div>
              ))}
            </div>
            <div className="flex items-center justify-between border-t border-void-700 pt-3">
              <span className="text-xs font-semibold uppercase tracking-wide text-void-500">
                Total outstanding
              </span>
              <span className="font-mono text-sm font-semibold text-void-50">
                {formatCurrency(total.toFixed(2))}
              </span>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
