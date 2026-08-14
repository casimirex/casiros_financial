import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Scale } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { taxApi } from "@/api/tax";
import type { DeferredTaxPosition } from "@/api/types";
import { formatCurrency } from "@/lib/format";

function describePosition(position: DeferredTaxPosition): { kind: "None" | "Liability" | "Asset"; amount: string | null } {
  if (position === "None") return { kind: "None", amount: null };
  if ("Liability" in position) return { kind: "Liability", amount: position.Liability };
  return { kind: "Asset", amount: position.Asset };
}

export function DeferredTaxPanel() {
  const [description, setDescription] = useState("Fixed asset depreciation");
  const [bookBasis, setBookBasis] = useState("");
  const [taxBasis, setTaxBasis] = useState("");
  const [taxRatePercent, setTaxRatePercent] = useState("21");

  const positionMutation = useMutation({
    mutationFn: () =>
      taxApi.deferredPosition({
        description,
        book_basis: bookBasis,
        tax_basis: taxBasis,
        tax_rate: (Number(taxRatePercent) / 100).toString(),
      }),
  });

  const result = positionMutation.data;
  const position = result ? describePosition(result) : null;

  return (
    <Card className="max-w-2xl">
      <CardHeader>
        <CardTitle>Deferred tax position</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="space-y-1">
          <Label htmlFor="dt-description">Description</Label>
          <Input
            id="dt-description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
          />
        </div>
        <div className="grid grid-cols-3 gap-2">
          <div className="space-y-1">
            <Label htmlFor="dt-book">Book basis</Label>
            <Input id="dt-book" type="number" value={bookBasis} onChange={(e) => setBookBasis(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="dt-tax">Tax basis</Label>
            <Input id="dt-tax" type="number" value={taxBasis} onChange={(e) => setTaxBasis(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="dt-rate">Tax rate %</Label>
            <Input
              id="dt-rate"
              type="number"
              value={taxRatePercent}
              onChange={(e) => setTaxRatePercent(e.target.value)}
            />
          </div>
        </div>
        {positionMutation.isError && (
          <p className="text-xs text-unfavorable-500">{(positionMutation.error as Error).message}</p>
        )}
        <Button
          className="w-full"
          disabled={!bookBasis || !taxBasis || positionMutation.isPending}
          onClick={() => positionMutation.mutate()}
        >
          <Scale className="h-4 w-4" />
          Evaluate
        </Button>
        {position && (
          <div className="flex items-center justify-between rounded-lg border border-void-700 bg-void-900/60 p-4">
            <div>
              <div className="text-xs uppercase tracking-wide text-void-500">Position</div>
              <div className="font-mono text-2xl text-signal-400">
                {position.amount ? formatCurrency(position.amount) : "—"}
              </div>
            </div>
            <Badge
              variant={
                position.kind === "Liability"
                  ? "unfavorable"
                  : position.kind === "Asset"
                    ? "favorable"
                    : "neutral"
              }
            >
              {position.kind === "None" ? "No difference" : `Deferred tax ${position.kind.toLowerCase()}`}
            </Badge>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
