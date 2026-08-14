import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Plus, Sigma, X } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { taxApi, type JurisdictionAllocation } from "@/api/tax";
import { BracketsEditor } from "./brackets-editor";
import { bracketsFromForm, defaultBrackets, type BracketForm } from "./brackets";
import { formatCurrency } from "@/lib/format";

interface DraftAllocation {
  label: string;
  allocation: JurisdictionAllocation;
}

export function MultiJurisdictionPanel() {
  const [code, setCode] = useState("US-CA");
  const [name, setName] = useState("California");
  const [brackets, setBrackets] = useState<BracketForm[]>(defaultBrackets);
  const [taxableIncome, setTaxableIncome] = useState("");

  const [allocations, setAllocations] = useState<DraftAllocation[]>([]);

  const totalMutation = useMutation({
    mutationFn: () => taxApi.multiJurisdiction(allocations.map((a) => a.allocation)),
  });

  const addAllocation = () => {
    setAllocations((prev) => [
      ...prev,
      {
        label: `${name} (${code})`,
        allocation: {
          jurisdiction: { code, name, brackets: bracketsFromForm(brackets) },
          taxable_income: taxableIncome,
        },
      },
    ]);
    setTaxableIncome("");
  };

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[26rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Add jurisdiction allocation</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="mj-code">Code</Label>
              <Input id="mj-code" value={code} onChange={(e) => setCode(e.target.value)} />
            </div>
            <div className="space-y-1">
              <Label htmlFor="mj-name">Name</Label>
              <Input id="mj-name" value={name} onChange={(e) => setName(e.target.value)} />
            </div>
          </div>
          <BracketsEditor brackets={brackets} onChange={setBrackets} />
          <div className="space-y-1">
            <Label htmlFor="mj-income">Taxable income allocated here</Label>
            <Input
              id="mj-income"
              type="number"
              min="0"
              value={taxableIncome}
              onChange={(e) => setTaxableIncome(e.target.value)}
            />
          </div>
          <Button className="w-full" disabled={!name || !taxableIncome} onClick={addAllocation}>
            <Plus className="h-4 w-4" />
            Add allocation
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Allocations</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {allocations.length === 0 && (
            <p className="text-sm text-void-500">
              Add federal, state, and local allocations, then compute the combined tax.
            </p>
          )}
          <div className="divide-y divide-void-800">
            {allocations.map((a, i) => (
              <div key={i} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">{a.label}</div>
                  <div className="text-xs text-void-500">
                    income {formatCurrency(a.allocation.taxable_income)}
                  </div>
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setAllocations((prev) => prev.filter((_, idx) => idx !== i))}
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>
            ))}
          </div>

          {allocations.length > 0 && (
            <Button
              className="w-full"
              disabled={totalMutation.isPending}
              onClick={() => totalMutation.mutate()}
            >
              <Sigma className="h-4 w-4" />
              Compute total tax
            </Button>
          )}

          {totalMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(totalMutation.error as Error).message}</p>
          )}

          {totalMutation.data && (
            <div className="rounded-lg border border-void-700 bg-void-900/60 p-6 text-center">
              <div className="text-xs uppercase tracking-wide text-void-500">
                Total tax across {allocations.length} jurisdiction(s)
              </div>
              <div className="mt-1 font-mono text-3xl text-signal-400">
                {formatCurrency(totalMutation.data.total_tax)}
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
