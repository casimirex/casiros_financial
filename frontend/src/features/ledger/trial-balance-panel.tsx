import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ledgerApi } from "@/api/ledger";
import { formatCurrency, toNumber } from "@/lib/format";
import { cn } from "@/lib/utils";

export function TrialBalancePanel() {
  const balancesQuery = useQuery({ queryKey: ["trial-balance"], queryFn: ledgerApi.trialBalance });
  const accountsQuery = useQuery({ queryKey: ["accounts"], queryFn: ledgerApi.listAccounts });

  const nameFor = (code: number) => accountsQuery.data?.find((a) => a.code === code)?.name ?? "—";

  // A roll-up parent's balance is the sum of its children's (see
  // casiros_erp::ledger::consolidation), so summing every account only nets
  // to zero for leaf accounts — accounts that are themselves a parent would
  // double-count their children's balances.
  const parentCodes = new Set(
    accountsQuery.data?.map((a) => a.parent).filter((p): p is number => p !== null),
  );
  const leafTotal =
    balancesQuery.data
      ?.filter((b) => !parentCodes.has(b.code))
      .reduce((sum, b) => sum + toNumber(b.balance), 0) ?? 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Trial balance</CardTitle>
      </CardHeader>
      <CardContent>
        {balancesQuery.data?.length === 0 && (
          <p className="text-sm text-void-500">No accounts to report on yet.</p>
        )}
        <div className="divide-y divide-void-800">
          {balancesQuery.data?.map((balance) => {
            const value = toNumber(balance.balance);
            return (
              <div key={balance.code} className="flex items-center justify-between py-2.5">
                <div className="flex items-center gap-3">
                  <span className="font-mono text-sm text-void-400">{balance.code}</span>
                  <span className="text-sm text-void-100">{nameFor(balance.code)}</span>
                </div>
                <span
                  className={cn(
                    "font-mono text-sm",
                    value >= 0 ? "text-void-100" : "text-unfavorable-500",
                  )}
                >
                  {formatCurrency(balance.balance)}
                </span>
              </div>
            );
          })}
        </div>
        {balancesQuery.data && balancesQuery.data.length > 0 && (
          <div className="mt-2 flex items-center justify-between border-t border-void-700 pt-3">
            <span className="text-xs font-semibold uppercase tracking-wide text-void-500">
              Net of leaf accounts (should be zero)
            </span>
            <span
              className={cn(
                "font-mono text-sm font-semibold",
                Math.abs(leafTotal) < 0.01 ? "text-favorable-500" : "text-unfavorable-500",
              )}
            >
              {formatCurrency(leafTotal.toFixed(2))}
            </span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
