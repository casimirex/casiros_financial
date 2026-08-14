import { useMemo, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Sigma, TriangleAlert } from "lucide-react";
import { Shell } from "@/components/layout/shell";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { calculateApi } from "@/api/calculate";
import { ApiError } from "@/api/client";
import { FORMULA_CATEGORIES, FORMULAS } from "./formula-registry";

export function CalculatorPage() {
  const [formulaName, setFormulaName] = useState(FORMULAS[0].name);
  const [values, setValues] = useState<Record<string, string>>({});

  const formula = useMemo(() => FORMULAS.find((f) => f.name === formulaName)!, [formulaName]);

  const mutation = useMutation({
    mutationFn: () => calculateApi.evaluate(formula.name, values),
  });

  const handleFormulaChange = (name: string) => {
    setFormulaName(name);
    setValues({});
    mutation.reset();
  };

  return (
    <Shell title="Calculator" subtitle="Evaluate any of the 41 scalar core formulas directly.">
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Formula</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <Select value={formulaName} onValueChange={handleFormulaChange}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {FORMULA_CATEGORIES.map((category) => (
                  <div key={category}>
                    <div className="px-2 pb-1 pt-2 text-[10px] font-semibold uppercase tracking-wider text-void-500">
                      {category}
                    </div>
                    {FORMULAS.filter((f) => f.category === category).map((f) => (
                      <SelectItem key={f.name} value={f.name}>
                        {f.label}
                      </SelectItem>
                    ))}
                  </div>
                ))}
              </SelectContent>
            </Select>

            <div className="space-y-3">
              {formula.params.map((param) => (
                <div key={param} className="space-y-1">
                  <Label htmlFor={param}>{param.replace(/_/g, " ")}</Label>
                  <Input
                    id={param}
                    inputMode="decimal"
                    placeholder="0.00"
                    value={values[param] ?? ""}
                    onChange={(e) => setValues((v) => ({ ...v, [param]: e.target.value }))}
                  />
                </div>
              ))}
            </div>

            <Button
              className="w-full"
              onClick={() => mutation.mutate()}
              disabled={mutation.isPending || formula.params.some((p) => !values[p])}
            >
              <Sigma className="h-4 w-4" />
              {mutation.isPending ? "Computing..." : "Evaluate"}
            </Button>
          </CardContent>
        </Card>

        <Card className="flex items-center justify-center">
          <CardContent className="w-full py-16 text-center">
            {mutation.isIdle && (
              <p className="text-sm text-void-500">Fill in the parameters and evaluate.</p>
            )}
            {mutation.isError && (
              <div className="flex flex-col items-center gap-2 text-unfavorable-500">
                <TriangleAlert className="h-6 w-6" />
                <p className="text-sm">
                  {mutation.error instanceof ApiError
                    ? mutation.error.message
                    : "Something went wrong."}
                </p>
              </div>
            )}
            {mutation.isSuccess && (
              <div className="space-y-2">
                <div className="text-xs uppercase tracking-wider text-void-500">
                  {mutation.data.formula.replace(/_/g, " ")}
                </div>
                <div className="text-gradient-signal font-mono text-5xl font-bold">
                  {mutation.data.result}
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </Shell>
  );
}
