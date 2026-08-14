import { Shell } from "@/components/layout/shell";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { AccountsPanel } from "./accounts-panel";
import { JournalPanel } from "./journal-panel";
import { TrialBalancePanel } from "./trial-balance-panel";

export function LedgerPage() {
  return (
    <Shell title="Ledger" subtitle="The causal general ledger — chart of accounts, journal entries, trial balance.">
      <Tabs defaultValue="accounts">
        <TabsList>
          <TabsTrigger value="accounts">Accounts</TabsTrigger>
          <TabsTrigger value="journal">Journal</TabsTrigger>
          <TabsTrigger value="trial-balance">Trial Balance</TabsTrigger>
        </TabsList>
        <TabsContent value="accounts">
          <AccountsPanel />
        </TabsContent>
        <TabsContent value="journal">
          <JournalPanel />
        </TabsContent>
        <TabsContent value="trial-balance">
          <TrialBalancePanel />
        </TabsContent>
      </Tabs>
    </Shell>
  );
}
