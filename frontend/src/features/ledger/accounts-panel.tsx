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
import { ledgerApi } from "@/api/ledger";
import type { Account, AccountType } from "@/api/types";

const ACCOUNT_TYPES: AccountType[] = ["Asset", "Liability", "Equity", "Revenue", "Expense"];

const TYPE_VARIANT: Record<AccountType, "favorable" | "unfavorable" | "default" | "caution" | "neutral"> = {
  Asset: "favorable",
  Revenue: "favorable",
  Equity: "default",
  Liability: "caution",
  Expense: "unfavorable",
};

export function AccountsPanel() {
  const queryClient = useQueryClient();
  const accountsQuery = useQuery({ queryKey: ["accounts"], queryFn: ledgerApi.listAccounts });

  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [accountType, setAccountType] = useState<AccountType>("Asset");

  const createMutation = useMutation({
    mutationFn: (account: Account) => ledgerApi.registerAccount(account),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["accounts"] });
      setCode("");
      setName("");
    },
  });

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[20rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Register account</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-1">
            <Label htmlFor="acc-code">Code</Label>
            <Input id="acc-code" type="number" value={code} onChange={(e) => setCode(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="acc-name">Name</Label>
            <Input id="acc-name" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label>Type</Label>
            <Select value={accountType} onValueChange={(v) => setAccountType(v as AccountType)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {ACCOUNT_TYPES.map((t) => (
                  <SelectItem key={t} value={t}>
                    {t}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          {createMutation.isError && (
            <p className="text-xs text-unfavorable-500">
              {(createMutation.error as Error).message}
            </p>
          )}
          <Button
            className="w-full"
            disabled={!code || !name || createMutation.isPending}
            onClick={() =>
              createMutation.mutate({ code: Number(code), name, account_type: accountType, parent: null })
            }
          >
            <Plus className="h-4 w-4" />
            Register
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Chart of accounts</CardTitle>
        </CardHeader>
        <CardContent>
          {accountsQuery.isLoading && <p className="text-sm text-void-500">Loading...</p>}
          {accountsQuery.data && accountsQuery.data.length === 0 && (
            <p className="text-sm text-void-500">No accounts registered yet.</p>
          )}
          <div className="divide-y divide-void-800">
            {accountsQuery.data?.map((account) => (
              <div key={account.code} className="flex items-center justify-between py-3">
                <div className="flex items-center gap-3">
                  <span className="font-mono text-sm text-void-400">{account.code}</span>
                  <span className="text-sm text-void-100">{account.name}</span>
                </div>
                <Badge variant={TYPE_VARIANT[account.account_type]}>{account.account_type}</Badge>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
