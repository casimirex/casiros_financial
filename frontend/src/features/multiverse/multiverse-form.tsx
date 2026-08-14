import { useState } from "react";
import { ChevronDown, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type { MonteCarloConfig, SimulateRequest, Universe } from "@/api/types";
import {
  DEFAULT_CONFIG,
  DEFAULT_UNIVERSE,
  FIELD_LABELS,
  UNIVERSE_FIELD_GROUPS,
} from "./default-universe";

export function MultiverseForm({
  onRun,
  disabled,
}: {
  onRun: (request: SimulateRequest) => void;
  disabled: boolean;
}) {
  const [universe, setUniverse] = useState<Universe>(DEFAULT_UNIVERSE);
  const [config, setConfig] = useState<MonteCarloConfig>(DEFAULT_CONFIG);
  const [expanded, setExpanded] = useState(false);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Configure the simulation</CardTitle>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-1.5">
            <Label htmlFor="iterations">Scenarios</Label>
            <Input
              id="iterations"
              type="number"
              min={100}
              max={1_000_000}
              value={config.iterations}
              onChange={(e) => setConfig((c) => ({ ...c, iterations: Number(e.target.value) }))}
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="seed">Seed</Label>
            <Input
              id="seed"
              type="number"
              value={config.seed}
              onChange={(e) => setConfig((c) => ({ ...c, seed: Number(e.target.value) }))}
            />
          </div>
        </div>

        <button
          type="button"
          onClick={() => setExpanded((v) => !v)}
          className="flex w-full items-center justify-between text-xs font-medium uppercase tracking-wide text-void-400 hover:text-void-200"
        >
          Baseline universe parameters
          <ChevronDown className={cn("h-4 w-4 transition-transform", expanded && "rotate-180")} />
        </button>

        {expanded && (
          <div className="max-h-80 space-y-4 overflow-y-auto pr-1">
            {UNIVERSE_FIELD_GROUPS.map((group) => (
              <div key={group.title}>
                <div className="mb-2 text-[11px] font-semibold uppercase tracking-wider text-void-500">
                  {group.title}
                </div>
                <div className="grid grid-cols-2 gap-2.5">
                  {group.fields.map((field) => (
                    <div key={field} className="space-y-1">
                      <Label htmlFor={field}>{FIELD_LABELS[field]}</Label>
                      <Input
                        id={field}
                        value={universe[field]}
                        onChange={(e) => setUniverse((u) => ({ ...u, [field]: e.target.value }))}
                      />
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        <Button
          className="w-full"
          size="lg"
          disabled={disabled}
          onClick={() => onRun({ baseline: universe, config })}
        >
          <Play className="h-4 w-4" />
          {disabled ? "Simulating..." : "Run Simulation"}
        </Button>
      </CardContent>
    </Card>
  );
}
