import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Shield } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { treasuryApi } from "@/api/treasury";
import { formatDecimalString } from "@/lib/format";

export function HedgePanel() {
  const [hedgeGainLoss, setHedgeGainLoss] = useState("");
  const [exposureGainLoss, setExposureGainLoss] = useState("");

  const effectivenessMutation = useMutation({
    mutationFn: () =>
      treasuryApi.hedgeEffectiveness({
        hedge_gain_loss: hedgeGainLoss,
        exposure_gain_loss: exposureGainLoss,
      }),
  });

  return (
    <Card className="max-w-2xl">
      <CardHeader>
        <CardTitle>Hedge effectiveness</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <p className="text-xs text-void-500">
          Dollar-offset method: qualifies as highly effective within the 80%–125% range.
        </p>
        <div className="grid grid-cols-2 gap-2">
          <div className="space-y-1">
            <Label htmlFor="hedge-gain">Hedge gain/(loss)</Label>
            <Input
              id="hedge-gain"
              type="number"
              value={hedgeGainLoss}
              onChange={(e) => setHedgeGainLoss(e.target.value)}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="exposure-gain">Exposure gain/(loss)</Label>
            <Input
              id="exposure-gain"
              type="number"
              value={exposureGainLoss}
              onChange={(e) => setExposureGainLoss(e.target.value)}
            />
          </div>
        </div>
        {effectivenessMutation.isError && (
          <p className="text-xs text-unfavorable-500">
            {(effectivenessMutation.error as Error).message}
          </p>
        )}
        <Button
          className="w-full"
          disabled={!hedgeGainLoss || !exposureGainLoss || effectivenessMutation.isPending}
          onClick={() => effectivenessMutation.mutate()}
        >
          <Shield className="h-4 w-4" />
          Evaluate
        </Button>
        {effectivenessMutation.data && (
          <div className="flex items-center justify-between rounded-lg border border-void-700 bg-void-900/60 p-4">
            <div>
              <div className="text-xs uppercase tracking-wide text-void-500">Effectiveness ratio</div>
              <div className="font-mono text-2xl text-signal-400">
                {formatDecimalString(effectivenessMutation.data.effectiveness, 4)}
              </div>
            </div>
            <Badge variant={effectivenessMutation.data.highly_effective ? "favorable" : "unfavorable"}>
              {effectivenessMutation.data.highly_effective ? "Highly effective" : "Not effective"}
            </Badge>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
