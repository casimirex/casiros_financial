import { Shell } from "@/components/layout/shell";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { CalculatorPanel } from "./calculator-panel";
import { MultiJurisdictionPanel } from "./multi-jurisdiction-panel";
import { DeferredTaxPanel } from "./deferred-tax-panel";

export function TaxPage() {
  return (
    <Shell
      title="Tax"
      subtitle="Progressive tax calculation, multi-jurisdiction aggregation, and deferred tax."
    >
      <Tabs defaultValue="calculator">
        <TabsList>
          <TabsTrigger value="calculator">Calculator</TabsTrigger>
          <TabsTrigger value="multi-jurisdiction">Multi-jurisdiction</TabsTrigger>
          <TabsTrigger value="deferred">Deferred tax</TabsTrigger>
        </TabsList>
        <TabsContent value="calculator">
          <CalculatorPanel />
        </TabsContent>
        <TabsContent value="multi-jurisdiction">
          <MultiJurisdictionPanel />
        </TabsContent>
        <TabsContent value="deferred">
          <DeferredTaxPanel />
        </TabsContent>
      </Tabs>
    </Shell>
  );
}
