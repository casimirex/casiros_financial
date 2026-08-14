import { useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Send } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { apApi } from "@/api/ap";
import { formatCurrency, shortId } from "@/lib/format";

export function PaymentsPanel() {
  const suppliersQuery = useQuery({ queryKey: ["ap-suppliers"], queryFn: apApi.listSuppliers });
  const [asOf, setAsOf] = useState(new Date().toISOString().slice(0, 10));
  const [availableCash, setAvailableCash] = useState("");
  const [currentLiabilities, setCurrentLiabilities] = useState("");

  const proposeMutation = useMutation({
    mutationFn: () =>
      apApi.propose({
        as_of: asOf,
        available_cash: availableCash,
        current_liabilities: currentLiabilities,
      }),
  });

  const supplierName = (id: string) =>
    suppliersQuery.data?.find((s) => s.id === id)?.name ?? shortId(id);

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Propose payments</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1">
            <Label htmlFor="pp-as-of">As of</Label>
            <Input id="pp-as-of" type="date" value={asOf} onChange={(e) => setAsOf(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="pp-cash">Available cash</Label>
            <Input
              id="pp-cash"
              type="number"
              min="0"
              value={availableCash}
              onChange={(e) => setAvailableCash(e.target.value)}
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="pp-liabilities">Current liabilities</Label>
            <Input
              id="pp-liabilities"
              type="number"
              min="0"
              value={currentLiabilities}
              onChange={(e) => setCurrentLiabilities(e.target.value)}
            />
          </div>
          {proposeMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(proposeMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!availableCash || !currentLiabilities || proposeMutation.isPending}
            onClick={() => proposeMutation.mutate()}
          >
            <Send className="h-4 w-4" />
            Propose
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Proposed payments</CardTitle>
        </CardHeader>
        <CardContent>
          {!proposeMutation.data && (
            <p className="text-sm text-void-500">Run a proposal to see recommended payments here.</p>
          )}
          {proposeMutation.data?.length === 0 && (
            <p className="text-sm text-void-500">No payments proposed under these constraints.</p>
          )}
          <div className="divide-y divide-void-800">
            {proposeMutation.data?.map((proposal) => (
              <div key={proposal.supplier} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">{supplierName(proposal.supplier)}</div>
                  <div className="text-xs text-void-500">{proposal.invoices.length} invoice(s)</div>
                </div>
                <span className="font-mono text-sm text-favorable-500">
                  {formatCurrency(proposal.total_amount)}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
