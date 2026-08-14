import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { arApi } from "@/api/ar";
import { PaymentTermsFields } from "@/features/shared/payment-terms-fields";
import {
  emptyPaymentTermsForm,
  formatPaymentTerms,
  paymentTermsFromForm,
  type PaymentTermsForm,
} from "@/features/shared/payment-terms";
import { formatCurrency, shortId } from "@/lib/format";

export function CustomersPanel() {
  const queryClient = useQueryClient();
  const customersQuery = useQuery({ queryKey: ["ar-customers"], queryFn: arApi.listCustomers });

  const [name, setName] = useState("");
  const [creditLimit, setCreditLimit] = useState("");
  const [receivableAccount, setReceivableAccount] = useState("");
  const [terms, setTerms] = useState<PaymentTermsForm>(emptyPaymentTermsForm);

  const createMutation = useMutation({
    mutationFn: () =>
      arApi.createCustomer({
        name,
        credit_limit: creditLimit,
        payment_terms: paymentTermsFromForm(terms),
        receivable_account: Number(receivableAccount),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ar-customers"] });
      setName("");
      setCreditLimit("");
      setReceivableAccount("");
      setTerms(emptyPaymentTermsForm);
    },
  });

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Register customer</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1">
            <Label htmlFor="customer-name">Name</Label>
            <Input id="customer-name" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div className="space-y-1">
              <Label htmlFor="customer-credit-limit">Credit limit</Label>
              <Input
                id="customer-credit-limit"
                type="number"
                min="0"
                value={creditLimit}
                onChange={(e) => setCreditLimit(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="customer-account">Receivable acct</Label>
              <Input
                id="customer-account"
                type="number"
                value={receivableAccount}
                onChange={(e) => setReceivableAccount(e.target.value)}
              />
            </div>
          </div>
          <PaymentTermsFields idPrefix="customer" value={terms} onChange={setTerms} />
          {createMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(createMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!name || !creditLimit || !receivableAccount || createMutation.isPending}
            onClick={() => createMutation.mutate()}
          >
            <Plus className="h-4 w-4" />
            Register
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Customers</CardTitle>
        </CardHeader>
        <CardContent>
          {customersQuery.data?.length === 0 && (
            <p className="text-sm text-void-500">No customers registered yet.</p>
          )}
          <div className="divide-y divide-void-800">
            {customersQuery.data?.map((customer) => (
              <div key={customer.id} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">{customer.name}</div>
                  <div className="font-mono text-xs text-void-500">
                    {shortId(customer.id)} · limit {formatCurrency(customer.credit_limit)}
                  </div>
                </div>
                <span className="text-xs text-void-400">
                  {formatPaymentTerms(customer.payment_terms)}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
