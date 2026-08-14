import { Shell } from "@/components/layout/shell";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { CashflowPanel } from "./cashflow-panel";
import { FxPanel } from "./fx-panel";
import { HedgePanel } from "./hedge-panel";

export function TreasuryPage() {
  return (
    <Shell title="Treasury" subtitle="Cash forecasting, FX conversion, and hedge effectiveness.">
      <Tabs defaultValue="cashflow">
        <TabsList>
          <TabsTrigger value="cashflow">Cash forecast</TabsTrigger>
          <TabsTrigger value="fx">FX</TabsTrigger>
          <TabsTrigger value="hedge">Hedge effectiveness</TabsTrigger>
        </TabsList>
        <TabsContent value="cashflow">
          <CashflowPanel />
        </TabsContent>
        <TabsContent value="fx">
          <FxPanel />
        </TabsContent>
        <TabsContent value="hedge">
          <HedgePanel />
        </TabsContent>
      </Tabs>
    </Shell>
  );
}
