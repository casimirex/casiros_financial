import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { ArrowRightLeft } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { treasuryApi } from "@/api/treasury";
import { currencyCodeToBytes, formatDecimalString } from "@/lib/format";

export function FxPanel() {
  const [fromCurrency, setFromCurrency] = useState("USD");
  const [toCurrency, setToCurrency] = useState("EUR");
  const [amount, setAmount] = useState("");
  const [rate, setRate] = useState("");
  const [asOf, setAsOf] = useState(new Date().toISOString().slice(0, 10));

  const convertMutation = useMutation({
    mutationFn: () =>
      treasuryApi.convert({
        exposure: { currency: currencyCodeToBytes(fromCurrency), amount },
        rate: {
          from: currencyCodeToBytes(fromCurrency),
          to: currencyCodeToBytes(toCurrency),
          rate,
          as_of: asOf,
        },
      }),
  });

  return (
    <Card className="max-w-2xl">
      <CardHeader>
        <CardTitle>FX conversion</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="grid grid-cols-2 gap-2">
          <div className="space-y-1">
            <Label htmlFor="fx-from">From currency</Label>
            <Input
              id="fx-from"
              maxLength={3}
              value={fromCurrency}
              onChange={(e) => setFromCurrency(e.target.value.toUpperCase())}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="fx-to">To currency</Label>
            <Input
              id="fx-to"
              maxLength={3}
              value={toCurrency}
              onChange={(e) => setToCurrency(e.target.value.toUpperCase())}
            />
          </div>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <div className="space-y-1">
            <Label htmlFor="fx-amount">Exposure amount</Label>
            <Input id="fx-amount" type="number" value={amount} onChange={(e) => setAmount(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="fx-rate">Rate</Label>
            <Input id="fx-rate" type="number" step="0.0001" value={rate} onChange={(e) => setRate(e.target.value)} />
          </div>
        </div>
        <div className="space-y-1">
          <Label htmlFor="fx-as-of">As of</Label>
          <Input id="fx-as-of" type="date" value={asOf} onChange={(e) => setAsOf(e.target.value)} />
        </div>
        {convertMutation.isError && (
          <p className="text-xs text-unfavorable-500">{(convertMutation.error as Error).message}</p>
        )}
        <Button
          className="w-full"
          disabled={!amount || !rate || convertMutation.isPending}
          onClick={() => convertMutation.mutate()}
        >
          <ArrowRightLeft className="h-4 w-4" />
          Convert
        </Button>
        {convertMutation.data && (
          <div className="rounded-lg border border-void-700 bg-void-900/60 p-4 text-center">
            <div className="text-xs uppercase tracking-wide text-void-500">Converted amount</div>
            <div className="mt-1 font-mono text-2xl text-signal-400">
              {formatDecimalString(convertMutation.data.converted)} {toCurrency}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
