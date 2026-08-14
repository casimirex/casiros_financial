import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
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
import { apApi } from "@/api/ap";
import type { ApInvoiceStatus } from "@/api/types";
import { PaymentTermsFields } from "@/features/shared/payment-terms-fields";
import {
  emptyPaymentTermsForm,
  paymentTermsFromForm,
  type PaymentTermsForm,
} from "@/features/shared/payment-terms";
import { formatCurrency, shortId, toNumber } from "@/lib/format";

const STATUS_VARIANT: Record<ApInvoiceStatus, "favorable" | "caution" | "neutral"> = {
  Paid: "favorable",
  PartiallyPaid: "caution",
  Open: "neutral",
};

export function InvoicesPanel() {
  const queryClient = useQueryClient();
  const suppliersQuery = useQuery({ queryKey: ["ap-suppliers"], queryFn: apApi.listSuppliers });
  const invoicesQuery = useQuery({ queryKey: ["ap-invoices"], queryFn: apApi.listInvoices });

  const [supplierId, setSupplierId] = useState("");
  const [invoiceNumber, setInvoiceNumber] = useState("");
  const [invoiceDate, setInvoiceDate] = useState(new Date().toISOString().slice(0, 10));
  const [amount, setAmount] = useState("");
  const [terms, setTerms] = useState<PaymentTermsForm>(emptyPaymentTermsForm);

  const createMutation = useMutation({
    mutationFn: () =>
      apApi.createInvoice({
        supplier: supplierId,
        invoice_number: invoiceNumber,
        invoice_date: invoiceDate,
        amount,
        terms: paymentTermsFromForm(terms),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ap-invoices"] });
      setInvoiceNumber("");
      setAmount("");
    },
  });

  const supplierName = (id: string) =>
    suppliersQuery.data?.find((s) => s.id === id)?.name ?? shortId(id);

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Record AP invoice</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1">
            <Label>Supplier</Label>
            <Select value={supplierId} onValueChange={setSupplierId}>
              <SelectTrigger>
                <SelectValue placeholder="Select supplier" />
              </SelectTrigger>
              <SelectContent>
                {suppliersQuery.data?.map((s) => (
                  <SelectItem key={s.id} value={s.id}>
                    {s.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1">
            <Label htmlFor="ap-inv-number">Invoice number</Label>
            <Input
              id="ap-inv-number"
              value={invoiceNumber}
              onChange={(e) => setInvoiceNumber(e.target.value)}
            />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="ap-inv-date">Invoice date</Label>
              <Input
                id="ap-inv-date"
                type="date"
                value={invoiceDate}
                onChange={(e) => setInvoiceDate(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="ap-inv-amount">Amount</Label>
              <Input
                id="ap-inv-amount"
                type="number"
                min="0"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </div>
          </div>
          <PaymentTermsFields idPrefix="ap-inv" value={terms} onChange={setTerms} />
          {createMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(createMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!supplierId || !invoiceNumber || !amount || createMutation.isPending}
            onClick={() => createMutation.mutate()}
          >
            <Plus className="h-4 w-4" />
            Record invoice
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>AP invoices</CardTitle>
        </CardHeader>
        <CardContent>
          {invoicesQuery.data?.length === 0 && (
            <p className="text-sm text-void-500">No AP invoices recorded yet.</p>
          )}
          <div className="divide-y divide-void-800">
            {invoicesQuery.data?.map((invoice) => (
              <div key={invoice.id} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">
                    {invoice.invoice_number}{" "}
                    <span className="text-void-500">· {supplierName(invoice.supplier)}</span>
                  </div>
                  <div className="text-xs text-void-500">
                    issued {invoice.invoice_date} · balance{" "}
                    {formatCurrency((toNumber(invoice.amount) - toNumber(invoice.amount_paid)).toFixed(2))}
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <span className="font-mono text-sm text-void-100">
                    {formatCurrency(invoice.amount)}
                  </span>
                  <Badge variant={STATUS_VARIANT[invoice.status]}>{invoice.status}</Badge>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
