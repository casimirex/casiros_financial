import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { ArrowRight } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { causalityApi } from "@/api/causality";
import { pascalToSnakeCase, titleCase } from "@/lib/format";

export function FormulaGraphPanel() {
  const catalogQuery = useQuery({ queryKey: ["causality-formulas"], queryFn: causalityApi.listFormulas });
  const [selected, setSelected] = useState("dupont_roe");

  const graphQuery = useQuery({
    queryKey: ["causality-formula-graph", selected],
    queryFn: () => causalityApi.formulaGraph(selected),
    enabled: Boolean(selected),
  });

  const options = (catalogQuery.data ?? [])
    .map((f) => pascalToSnakeCase(f.formula))
    .sort((a, b) => a.localeCompare(b));

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Pick a formula</CardTitle>
          <CardDescription>
            Every formula's dependency wiring is real: the exact-name convention{" "}
            <code className="rounded bg-void-800 px-1 py-0.5 text-xs">
              casiros_dag::evaluator::resolve
            </code>{" "}
            uses to prefer a prior computed result over a raw input.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="max-w-xs space-y-1">
            <Label>Formula</Label>
            <Select value={selected} onValueChange={setSelected}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {options.map((name) => (
                  <SelectItem key={name} value={name}>
                    {titleCase(name)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      {graphQuery.data && (
        <Card>
          <CardHeader>
            <CardTitle>Dependency graph</CardTitle>
            <CardDescription>
              Evaluation order, left to right, via{" "}
              <code className="rounded bg-void-800 px-1 py-0.5 text-xs">
                CausalityEngine::execution_order
              </code>
              .
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap items-stretch gap-3">
              {graphQuery.data.nodes.map((node, i) => {
                const nodeName = pascalToSnakeCase(node.formula);
                const isTarget = node.formula === graphQuery.data.target;
                return (
                  <div key={node.formula} className="flex items-center gap-3">
                    <div
                      className={
                        isTarget
                          ? "min-w-[14rem] rounded-lg border border-signal-500/50 bg-signal-500/10 p-4"
                          : "min-w-[14rem] rounded-lg border border-void-700 bg-void-900/60 p-4"
                      }
                    >
                      <div className="mb-2 flex items-center justify-between">
                        <span className="text-sm font-semibold text-void-50">{titleCase(nodeName)}</span>
                        {isTarget && <Badge>target</Badge>}
                      </div>
                      <div className="space-y-1">
                        {node.parameters.map((param) => (
                          <div key={param.name} className="flex items-center justify-between gap-2">
                            <span className="font-mono text-xs text-void-400">{param.name}</span>
                            {param.upstream_formula ? (
                              <Badge variant="default" className="whitespace-nowrap">
                                {titleCase(pascalToSnakeCase(param.upstream_formula))}
                              </Badge>
                            ) : (
                              <Badge variant="neutral">input</Badge>
                            )}
                          </div>
                        ))}
                      </div>
                    </div>
                    {i < graphQuery.data.nodes.length - 1 && (
                      <ArrowRight className="h-5 w-5 shrink-0 text-void-600" />
                    )}
                  </div>
                );
              })}
            </div>
            {graphQuery.data.nodes.length === 1 && (
              <p className="mt-4 text-sm text-void-500">
                No other formula's output matches one of this formula's parameter names — every
                input here is raw.
              </p>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
