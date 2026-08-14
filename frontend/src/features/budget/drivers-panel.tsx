import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { budgetApi } from "@/api/budget";
import { formatDecimalString } from "@/lib/format";
import type { DriversState } from "./use-drivers";

export function DriversPanel({ state }: { state: DriversState }) {
  const [name, setName] = useState("");
  const [value, setValue] = useState("");

  const setMutation = useMutation({
    mutationFn: (request: { name: string; value: string }) => budgetApi.setDriver(request),
    onSuccess: (driver) => {
      state.setDriver(driver.name, driver.value);
      setName("");
      setValue("");
    },
  });

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Set driver</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1">
            <Label htmlFor="driver-name">Driver name</Label>
            <Input
              id="driver-name"
              placeholder="units_sold"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="driver-value">Value</Label>
            <Input id="driver-value" type="number" value={value} onChange={(e) => setValue(e.target.value)} />
          </div>
          {setMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(setMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!name || !value || setMutation.isPending}
            onClick={() => setMutation.mutate({ name, value })}
          >
            <Plus className="h-4 w-4" />
            Set driver
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Drivers (this session)</CardTitle>
        </CardHeader>
        <CardContent>
          {Object.keys(state.drivers).length === 0 && (
            <p className="text-sm text-void-500">No drivers set yet.</p>
          )}
          <div className="divide-y divide-void-800">
            {Object.entries(state.drivers).map(([driverName, driverValue]) => (
              <div key={driverName} className="flex items-center justify-between py-3">
                <span className="font-mono text-sm text-void-100">{driverName}</span>
                <span className="font-mono text-sm text-signal-400">
                  {formatDecimalString(driverValue)}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
