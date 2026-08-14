import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { budgetApi } from "@/api/budget";
import { formatCurrency } from "@/lib/format";
import { cn } from "@/lib/utils";
import type { DriversState } from "./use-drivers";

export function LineItemsPanel({ state }: { state: DriversState }) {
  const queryClient = useQueryClient();
  const lineItemsQuery = useQuery({ queryKey: ["budget-line-items"], queryFn: budgetApi.listLineItems });
  const totalQuery = useQuery({
    queryKey: ["budget-total"],
    queryFn: budgetApi.total,
    enabled: (lineItemsQuery.data?.length ?? 0) > 0,
  });

  const [account, setAccount] = useState("");
  const [description, setDescription] = useState("");
  const [selectedDrivers, setSelectedDrivers] = useState<string[]>([]);

  const toggleDriver = (name: string) => {
    setSelectedDrivers((prev) =>
      prev.includes(name) ? prev.filter((d) => d !== name) : [...prev, name],
    );
  };

  const addMutation = useMutation({
    mutationFn: () =>
      budgetApi.addLineItem({
        account: Number(account),
        description,
        driver_names: selectedDrivers,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["budget-line-items"] });
      queryClient.invalidateQueries({ queryKey: ["budget-total"] });
      setAccount("");
      setDescription("");
      setSelectedDrivers([]);
    },
  });

  const driverNames = Object.keys(state.drivers);

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Add line item</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1">
            <Label htmlFor="li-account">Account code</Label>
            <Input id="li-account" type="number" value={account} onChange={(e) => setAccount(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="li-description">Description</Label>
            <Input
              id="li-description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>
          <div className="space-y-1">
            <Label>Drivers (product of selected)</Label>
            {driverNames.length === 0 ? (
              <p className="text-xs text-void-500">Set a driver in the Drivers tab first.</p>
            ) : (
              <div className="flex flex-wrap gap-2">
                {driverNames.map((driverName) => (
                  <button
                    key={driverName}
                    type="button"
                    onClick={() => toggleDriver(driverName)}
                    className={cn(
                      "rounded-full border px-2.5 py-0.5 font-mono text-xs transition-colors",
                      selectedDrivers.includes(driverName)
                        ? "border-signal-500/50 bg-signal-500/10 text-signal-400"
                        : "border-void-600 bg-void-800 text-void-400 hover:text-void-100",
                    )}
                  >
                    {driverName}
                  </button>
                ))}
              </div>
            )}
          </div>
          {addMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(addMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!account || !description || selectedDrivers.length === 0 || addMutation.isPending}
            onClick={() => addMutation.mutate()}
          >
            <Plus className="h-4 w-4" />
            Add line item
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex-row items-center justify-between">
          <CardTitle>Line items</CardTitle>
          {totalQuery.data && (
            <span className="font-mono text-sm text-signal-400">
              total {formatCurrency(totalQuery.data.total)}
            </span>
          )}
        </CardHeader>
        <CardContent>
          {lineItemsQuery.data?.length === 0 && (
            <p className="text-sm text-void-500">No line items yet.</p>
          )}
          <div className="divide-y divide-void-800">
            {lineItemsQuery.data?.map((item, i) => (
              <div key={i} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">{item.description}</div>
                  <div className="text-xs text-void-500">acct {item.account}</div>
                </div>
                <span className="font-mono text-xs text-void-400">
                  {item.driver_names.join(" × ")}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
