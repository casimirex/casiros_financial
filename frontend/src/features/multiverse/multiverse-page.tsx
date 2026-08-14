import { useMemo, useState } from "react";
import { Shell } from "@/components/layout/shell";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import type { MetricKey, SimulateRequest } from "@/api/types";
import { useSimulationStream } from "./use-simulation-stream";
import { MultiverseForm } from "./multiverse-form";
import { ScenarioField, type ScenarioPoint } from "./scenario-field";
import { AxisPicker, type AxisSelection } from "./axis-picker";
import { MetricSummary } from "./metric-summary";
import { sampleFromPercentiles, normalizeToRange } from "./percentile-sampling";
import { favorabilityScores } from "./favorability";

const POINT_COUNT = 1500;
const AXIS_SCALE = 5;

const DEFAULT_AXES: AxisSelection = {
  x: "return_on_equity",
  y: "sharpe_ratio",
  z: "wacc",
  color: "profit_margin",
};

function usePointCloud(
  metrics: Record<MetricKey, import("@/api/types").SimulationResults> | null,
  axes: AxisSelection,
): ScenarioPoint[] {
  return useMemo(() => {
    if (!metrics) return [];
    const xRaw = sampleFromPercentiles(metrics[axes.x], POINT_COUNT);
    const yRaw = sampleFromPercentiles(metrics[axes.y], POINT_COUNT);
    const zRaw = sampleFromPercentiles(metrics[axes.z], POINT_COUNT);
    const colorRaw = sampleFromPercentiles(metrics[axes.color], POINT_COUNT);

    const x = normalizeToRange(xRaw, AXIS_SCALE);
    const y = normalizeToRange(yRaw, AXIS_SCALE);
    const z = normalizeToRange(zRaw, AXIS_SCALE);
    const favorability = favorabilityScores(colorRaw, axes.color);

    return x.map((xi, i) => ({ x: xi, y: y[i], z: z[i], favorability: favorability[i] }));
  }, [metrics, axes]);
}

export function MultiversePage() {
  const stream = useSimulationStream();
  const [axes, setAxes] = useState<AxisSelection>(DEFAULT_AXES);
  const points = usePointCloud(stream.metrics, axes);

  const isRunning = stream.status === "connecting" || stream.status === "running";
  const progressPct = stream.total > 0 ? (stream.completed / stream.total) * 100 : 0;

  const handleRun = (request: SimulateRequest) => stream.run(request);

  return (
    <Shell
      title="The Multiverse"
      subtitle="Every point is one simulated financial future. Drag to look at the distribution from another angle."
    >
      <div className="grid grid-cols-1 gap-6 xl:grid-cols-[22rem_1fr_20rem]">
        <div className="space-y-6">
          <MultiverseForm onRun={handleRun} disabled={isRunning} />

          {stream.status !== "idle" && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  Simulation status
                  <Badge
                    variant={
                      stream.status === "done"
                        ? "favorable"
                        : stream.status === "error"
                          ? "unfavorable"
                          : "default"
                    }
                  >
                    {stream.status}
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {isRunning && (
                  <>
                    <Progress value={progressPct} />
                    <p className="font-mono text-xs text-void-400">
                      {stream.completed.toLocaleString()} / {stream.total.toLocaleString()} scenarios
                    </p>
                  </>
                )}
                {stream.status === "error" && (
                  <p className="text-sm text-unfavorable-500">{stream.errorMessage}</p>
                )}
                {stream.status === "done" && (
                  <p className="text-sm text-void-300">
                    {POINT_COUNT.toLocaleString()} points reconstructed from the returned
                    distribution — see axis captions below.
                  </p>
                )}
              </CardContent>
            </Card>
          )}
        </div>

        <Card className="h-[32rem] overflow-hidden p-0">
          {points.length > 0 ? (
            <ScenarioField
              points={points}
              axisLabels={{
                x: axes.x.replace(/_/g, " "),
                y: axes.y.replace(/_/g, " "),
                z: axes.z.replace(/_/g, " "),
              }}
            />
          ) : (
            <div className="flex h-full min-h-[32rem] flex-col items-center justify-center gap-3 text-center text-void-500">
              <div className="h-16 w-16 animate-pulse-slow rounded-full border border-dashed border-void-600" />
              <p className="max-w-xs text-sm">
                Configure the baseline and run a simulation — the field populates once results
                arrive.
              </p>
            </div>
          )}
        </Card>

        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Axes</CardTitle>
            </CardHeader>
            <CardContent>
              <AxisPicker selection={axes} onChange={setAxes} />
            </CardContent>
          </Card>

          {stream.metrics && (
            <Card>
              <CardHeader>
                <CardTitle>Distribution</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <MetricSummary
                  metricKey={axes.x}
                  stats={stream.metrics[axes.x]}
                  accentClassName="from-signal-600 to-signal-400"
                />
                <MetricSummary
                  metricKey={axes.y}
                  stats={stream.metrics[axes.y]}
                  accentClassName="from-nova-600 to-nova-400"
                />
                <MetricSummary
                  metricKey={axes.z}
                  stats={stream.metrics[axes.z]}
                  accentClassName="from-caution-500 to-caution-500"
                />
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </Shell>
  );
}
