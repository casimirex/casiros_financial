import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Send } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { arApi } from "@/api/ar";
import { formatCurrency, shortId } from "@/lib/format";

export function ReceiptsPanel() {
  const queryClient = useQueryClient();
  const customersQuery = useQuery({ queryKey: ["ar-customers"], queryFn: arApi.listCustomers });

  const [customerId, setCustomerId] = useState("");
  const [amount, setAmount] = useState("");
  const [date, setDate] = useState(new Date().toISOString().slice(0, 10));

  const allocateMutation = useMutation({
    mutationFn: () => arApi.allocateReceipt({ customer: customerId, amount, date }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ar-invoices"] });
      setAmount("");
    },
  });

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Allocate cash receipt</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1">
            <Label>Customer</Label>
            <Select value={customerId} onValueChange={setCustomerId}>
              <SelectTrigger>
                <SelectValue placeholder="Select customer" />
              </SelectTrigger>
              <SelectContent>
                {customersQuery.data?.map((c) => (
                  <SelectItem key={c.id} value={c.id}>
                    {c.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="receipt-amount">Amount received</Label>
              <Input
                id="receipt-amount"
                type="number"
                min="0"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="receipt-date">Date</Label>
              <Input
                id="receipt-date"
                type="date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
              />
            </div>
          </div>
          <p className="text-xs text-void-500">
            Applied oldest-due-first across this customer's open invoices.
          </p>
          {allocateMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(allocateMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!customerId || !amount || allocateMutation.isPending}
            onClick={() => allocateMutation.mutate()}
          >
            <Send className="h-4 w-4" />
            Allocate
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Allocation result</CardTitle>
        </CardHeader>
        <CardContent>
          {!allocateMutation.data && (
            <p className="text-sm text-void-500">
              Allocate a receipt to see how it was applied across invoices.
            </p>
          )}
          {allocateMutation.data?.length === 0 && (
            <p className="text-sm text-void-500">No open invoices to apply this receipt to.</p>
          )}
          <div className="divide-y divide-void-800">
            {allocateMutation.data?.map((allocation) => (
              <div key={allocation.invoice} className="flex items-center justify-between py-3">
                <span className="font-mono text-xs text-void-400">
                  {shortId(allocation.invoice)}
                </span>
                <span className="font-mono text-sm text-favorable-500">
                  {formatCurrency(allocation.amount_applied)}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
