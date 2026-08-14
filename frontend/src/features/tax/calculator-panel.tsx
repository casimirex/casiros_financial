import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Calculator } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { taxApi } from "@/api/tax";
import { BracketsEditor } from "./brackets-editor";
import { bracketsFromForm, defaultBrackets, type BracketForm } from "./brackets";
import { formatCurrency } from "@/lib/format";

export function CalculatorPanel() {
  const [jurisdictionCode, setJurisdictionCode] = useState("US-FEDERAL");
  const [jurisdictionName, setJurisdictionName] = useState("US Federal");
  const [brackets, setBrackets] = useState<BracketForm[]>(defaultBrackets);
  const [taxableIncome, setTaxableIncome] = useState("");

  const calculateMutation = useMutation({
    mutationFn: () =>
      taxApi.calculate({
        jurisdiction: {
          code: jurisdictionCode,
          name: jurisdictionName,
          brackets: bracketsFromForm(brackets),
        },
        taxable_income: taxableIncome,
      }),
  });

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[26rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Progressive tax calculator</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="tax-code">Jurisdiction code</Label>
              <Input
                id="tax-code"
                value={jurisdictionCode}
                onChange={(e) => setJurisdictionCode(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="tax-name">Name</Label>
              <Input
                id="tax-name"
                value={jurisdictionName}
                onChange={(e) => setJurisdictionName(e.target.value)}
              />
            </div>
          </div>
          <BracketsEditor brackets={brackets} onChange={setBrackets} />
          <div className="space-y-1">
            <Label htmlFor="tax-income">Taxable income</Label>
            <Input
              id="tax-income"
              type="number"
              min="0"
              value={taxableIncome}
              onChange={(e) => setTaxableIncome(e.target.value)}
            />
          </div>
          {calculateMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(calculateMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!taxableIncome || calculateMutation.isPending}
            onClick={() => calculateMutation.mutate()}
          >
            <Calculator className="h-4 w-4" />
            Calculate
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Result</CardTitle>
        </CardHeader>
        <CardContent>
          {!calculateMutation.data ? (
            <p className="text-sm text-void-500">
              Define a bracket schedule and taxable income to compute tax owed.
            </p>
          ) : (
            <div className="space-y-4">
              <div className="rounded-lg border border-void-700 bg-void-900/60 p-6 text-center">
                <div className="text-xs uppercase tracking-wide text-void-500">Tax owed</div>
                <div className="mt-1 font-mono text-3xl text-signal-400">
                  {formatCurrency(calculateMutation.data.tax)}
                </div>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-void-500">Effective rate</span>
                <span className="font-mono text-void-100">
                  {taxableIncome && Number(taxableIncome) > 0
                    ? `${((Number(calculateMutation.data.tax) / Number(taxableIncome)) * 100).toFixed(2)}%`
                    : "—"}
                </span>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
