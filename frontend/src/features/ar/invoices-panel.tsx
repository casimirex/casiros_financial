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
import { arApi } from "@/api/ar";
import type { ArInvoiceStatus, RecognitionMethod } from "@/api/types";
import { PaymentTermsFields } from "@/features/shared/payment-terms-fields";
import {
  emptyPaymentTermsForm,
  paymentTermsFromForm,
  type PaymentTermsForm,
} from "@/features/shared/payment-terms";
import { formatCurrency, shortId, toNumber } from "@/lib/format";

const STATUS_VARIANT: Record<ArInvoiceStatus, "favorable" | "caution" | "neutral"> = {
  Collected: "favorable",
  PartiallyCollected: "caution",
  Open: "neutral",
};

function describeRecognition(method: RecognitionMethod): string {
  if ("PointInTime" in method) return `at ${method.PointInTime.recognition_date}`;
  return `ratably ${method.RatablyOverTime.start} → ${method.RatablyOverTime.end}`;
}

export function InvoicesPanel() {
  const queryClient = useQueryClient();
  const customersQuery = useQuery({ queryKey: ["ar-customers"], queryFn: arApi.listCustomers });
  const invoicesQuery = useQuery({ queryKey: ["ar-invoices"], queryFn: arApi.listInvoices });

  const [customerId, setCustomerId] = useState("");
  const [invoiceNumber, setInvoiceNumber] = useState("");
  const [invoiceDate, setInvoiceDate] = useState(new Date().toISOString().slice(0, 10));
  const [amount, setAmount] = useState("");
  const [terms, setTerms] = useState<PaymentTermsForm>(emptyPaymentTermsForm);
  const [recognitionKind, setRecognitionKind] = useState<"point-in-time" | "ratably">(
    "point-in-time",
  );
  const [recognitionDate, setRecognitionDate] = useState(invoiceDate);
  const [periodStart, setPeriodStart] = useState(invoiceDate);
  const [periodEnd, setPeriodEnd] = useState(invoiceDate);

  const createMutation = useMutation({
    mutationFn: () =>
      arApi.createInvoice({
        customer: customerId,
        invoice_number: invoiceNumber,
        invoice_date: invoiceDate,
        amount,
        terms: paymentTermsFromForm(terms),
        recognition_method:
          recognitionKind === "point-in-time"
            ? { PointInTime: { recognition_date: recognitionDate } }
            : { RatablyOverTime: { start: periodStart, end: periodEnd } },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ar-invoices"] });
      setInvoiceNumber("");
      setAmount("");
    },
  });

  const customerName = (id: string) =>
    customersQuery.data?.find((c) => c.id === id)?.name ?? shortId(id);

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Record AR invoice</CardTitle>
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
          <div className="space-y-1">
            <Label htmlFor="ar-inv-number">Invoice number</Label>
            <Input
              id="ar-inv-number"
              value={invoiceNumber}
              onChange={(e) => setInvoiceNumber(e.target.value)}
            />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="ar-inv-date">Invoice date</Label>
              <Input
                id="ar-inv-date"
                type="date"
                value={invoiceDate}
                onChange={(e) => setInvoiceDate(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="ar-inv-amount">Amount</Label>
              <Input
                id="ar-inv-amount"
                type="number"
                min="0"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </div>
          </div>
          <PaymentTermsFields idPrefix="ar-inv" value={terms} onChange={setTerms} />

          <div className="space-y-1">
            <Label>Revenue recognition</Label>
            <Select
              value={recognitionKind}
              onValueChange={(v) => setRecognitionKind(v as "point-in-time" | "ratably")}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="point-in-time">Point in time</SelectItem>
                <SelectItem value="ratably">Ratably over time</SelectItem>
              </SelectContent>
            </Select>
          </div>
          {recognitionKind === "point-in-time" ? (
            <div className="space-y-1">
              <Label htmlFor="ar-recognition-date">Recognition date</Label>
              <Input
                id="ar-recognition-date"
                type="date"
                value={recognitionDate}
                onChange={(e) => setRecognitionDate(e.target.value)}
              />
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-2">
              <div className="space-y-1">
                <Label htmlFor="ar-period-start">Period start</Label>
                <Input
                  id="ar-period-start"
                  type="date"
                  value={periodStart}
                  onChange={(e) => setPeriodStart(e.target.value)}
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="ar-period-end">Period end</Label>
                <Input
                  id="ar-period-end"
                  type="date"
                  value={periodEnd}
                  onChange={(e) => setPeriodEnd(e.target.value)}
                />
              </div>
            </div>
          )}

          {createMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(createMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!customerId || !invoiceNumber || !amount || createMutation.isPending}
            onClick={() => createMutation.mutate()}
          >
            <Plus className="h-4 w-4" />
            Record invoice
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>AR invoices</CardTitle>
        </CardHeader>
        <CardContent>
          {invoicesQuery.data?.length === 0 && (
            <p className="text-sm text-void-500">No AR invoices recorded yet.</p>
          )}
          <div className="divide-y divide-void-800">
            {invoicesQuery.data?.map((invoice) => (
              <div key={invoice.id} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">
                    {invoice.invoice_number}{" "}
                    <span className="text-void-500">· {customerName(invoice.customer)}</span>
                  </div>
                  <div className="text-xs text-void-500">
                    balance{" "}
                    {formatCurrency(
                      (toNumber(invoice.amount) - toNumber(invoice.amount_received)).toFixed(2),
                    )}{" "}
                    · recognized {describeRecognition(invoice.recognition_method)}
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
