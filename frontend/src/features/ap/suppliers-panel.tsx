import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { apApi } from "@/api/ap";
import { PaymentTermsFields } from "@/features/shared/payment-terms-fields";
import {
  emptyPaymentTermsForm,
  formatPaymentTerms,
  paymentTermsFromForm,
  type PaymentTermsForm,
} from "@/features/shared/payment-terms";
import { shortId } from "@/lib/format";

export function SuppliersPanel() {
  const queryClient = useQueryClient();
  const suppliersQuery = useQuery({ queryKey: ["ap-suppliers"], queryFn: apApi.listSuppliers });

  const [name, setName] = useState("");
  const [payableAccount, setPayableAccount] = useState("");
  const [terms, setTerms] = useState<PaymentTermsForm>(emptyPaymentTermsForm);

  const createMutation = useMutation({
    mutationFn: () =>
      apApi.createSupplier({
        name,
        payment_terms: paymentTermsFromForm(terms),
        payable_account: Number(payableAccount),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["ap-suppliers"] });
      setName("");
      setPayableAccount("");
      setTerms(emptyPaymentTermsForm);
    },
  });

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Register supplier</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1">
            <Label htmlFor="supplier-name">Name</Label>
            <Input id="supplier-name" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="supplier-account">Payable account code</Label>
            <Input
              id="supplier-account"
              type="number"
              value={payableAccount}
              onChange={(e) => setPayableAccount(e.target.value)}
            />
          </div>
          <PaymentTermsFields idPrefix="supplier" value={terms} onChange={setTerms} />
          {createMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(createMutation.error as Error).message}</p>
          )}
          <Button
            className="w-full"
            disabled={!name || !payableAccount || createMutation.isPending}
            onClick={() => createMutation.mutate()}
          >
            <Plus className="h-4 w-4" />
            Register
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Suppliers</CardTitle>
        </CardHeader>
        <CardContent>
          {suppliersQuery.data?.length === 0 && (
            <p className="text-sm text-void-500">No suppliers registered yet.</p>
          )}
          <div className="divide-y divide-void-800">
            {suppliersQuery.data?.map((supplier) => (
              <div key={supplier.id} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">{supplier.name}</div>
                  <div className="font-mono text-xs text-void-500">
                    {shortId(supplier.id)} · payable acct {supplier.payable_account}
                  </div>
                </div>
                <span className="text-xs text-void-400">
                  {formatPaymentTerms(supplier.payment_terms)}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
