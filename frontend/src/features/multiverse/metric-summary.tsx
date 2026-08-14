import type { MetricKey, SimulationResults } from "@/api/types";
import { toNumber } from "@/lib/format";
import { cn } from "@/lib/utils";
import { METRIC_LABELS } from "./metric-labels";

function pct(value: number, min: number, max: number): number {
  const span = max - min || 1;
  return Math.min(100, Math.max(0, ((value - min) / span) * 100));
}

export function MetricSummary({
  metricKey,
  stats,
  accentClassName,
}: {
  metricKey: MetricKey;
  stats: SimulationResults;
  accentClassName?: string;
}) {
  const min = toNumber(stats.min);
  const max = toNumber(stats.max);
  const p5 = toNumber(stats.percentile_5);
  const p95 = toNumber(stats.percentile_95);
  const median = toNumber(stats.median);
  const mean = toNumber(stats.mean);

  return (
    <div className="space-y-1.5">
      <div className="flex items-baseline justify-between">
        <span className="text-xs font-medium text-void-300">{METRIC_LABELS[metricKey]}</span>
        <span className="font-mono text-xs text-void-400">
          median {median.toFixed(3)} · μ {mean.toFixed(3)}
        </span>
      </div>
      <div className="relative h-2 rounded-full bg-void-800">
        <div
          className={cn("absolute h-2 rounded-full bg-gradient-to-r opacity-70", accentClassName)}
          style={{ left: `${pct(p5, min, max)}%`, right: `${100 - pct(p95, min, max)}%` }}
        />
        <div
          className="absolute top-1/2 h-3 w-0.5 -translate-y-1/2 bg-void-50"
          style={{ left: `${pct(median, min, max)}%` }}
        />
      </div>
      <div className="flex justify-between font-mono text-[10px] text-void-500">
        <span>{min.toFixed(2)}</span>
        <span>p5–p95</span>
        <span>{max.toFixed(2)}</span>
      </div>
    </div>
  );
}
