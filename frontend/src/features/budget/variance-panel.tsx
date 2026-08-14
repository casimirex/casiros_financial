import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { GitCompare, Plus, X } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { budgetApi } from "@/api/budget";
import type { AccountType, VarianceEntry } from "@/api/types";
import { formatCurrency, formatPercent } from "@/lib/format";

const ACCOUNT_TYPES: AccountType[] = ["Asset", "Liability", "Equity", "Revenue", "Expense"];

export function VariancePanel() {
  const [account, setAccount] = useState("");
  const [accountType, setAccountType] = useState<AccountType>("Revenue");
  const [budget, setBudget] = useState("");
  const [actual, setActual] = useState("");
  const [entries, setEntries] = useState<VarianceEntry[]>([]);

  const varianceMutation = useMutation({
    mutationFn: () => budgetApi.variance(entries),
  });

  const addEntry = () => {
    setEntries((prev) => [
      ...prev,
      { account: Number(account), account_type: accountType, budget, actual },
    ]);
    setAccount("");
    setBudget("");
    setActual("");
  };

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Add variance entry</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="var-account">Account code</Label>
              <Input id="var-account" type="number" value={account} onChange={(e) => setAccount(e.target.value)} />
            </div>
            <div className="space-y-1">
              <Label>Type</Label>
              <Select value={accountType} onValueChange={(v) => setAccountType(v as AccountType)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {ACCOUNT_TYPES.map((t) => (
                    <SelectItem key={t} value={t}>
                      {t}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="var-budget">Budget</Label>
              <Input id="var-budget" type="number" value={budget} onChange={(e) => setBudget(e.target.value)} />
            </div>
            <div className="space-y-1">
              <Label htmlFor="var-actual">Actual</Label>
              <Input id="var-actual" type="number" value={actual} onChange={(e) => setActual(e.target.value)} />
            </div>
          </div>
          <Button
            variant="secondary"
            className="w-full"
            disabled={!account || !budget || !actual}
            onClick={addEntry}
          >
            <Plus className="h-4 w-4" />
            Add entry
          </Button>

          <div className="divide-y divide-void-800 border-t border-void-800 pt-2">
            {entries.map((entry, i) => (
              <div key={i} className="flex items-center justify-between py-2">
                <span className="font-mono text-xs text-void-400">
                  {entry.account} · {entry.account_type}
                </span>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => setEntries((prev) => prev.filter((_, idx) => idx !== i))}
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>
            ))}
          </div>

          {entries.length > 0 && (
            <Button
              className="w-full"
              disabled={varianceMutation.isPending}
              onClick={() => varianceMutation.mutate()}
            >
              <GitCompare className="h-4 w-4" />
              Analyze variance
            </Button>
          )}
          {varianceMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(varianceMutation.error as Error).message}</p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Variance results</CardTitle>
        </CardHeader>
        <CardContent>
          {!varianceMutation.data && (
            <p className="text-sm text-void-500">
              Add budget-versus-actual entries, then analyze to see favorable/unfavorable variance.
            </p>
          )}
          <div className="divide-y divide-void-800">
            {varianceMutation.data?.map((result) => (
              <div key={result.account} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">Account {result.account}</div>
                  <div className="text-xs text-void-500">
                    {formatCurrency(result.budget)} budget → {formatCurrency(result.actual)} actual
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <span className="font-mono text-sm text-void-100">
                    {formatCurrency(result.variance)}
                    {result.variance_percent && ` (${formatPercent(result.variance_percent)})`}
                  </span>
                  <Badge variant={result.favorable ? "favorable" : "unfavorable"}>
                    {result.favorable ? "Favorable" : "Unfavorable"}
                  </Badge>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
