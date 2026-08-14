import { Shell } from "@/components/layout/shell";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DriversPanel } from "./drivers-panel";
import { useDrivers } from "./use-drivers";
import { LineItemsPanel } from "./line-items-panel";
import { VariancePanel } from "./variance-panel";

export function BudgetPage() {
  const driversState = useDrivers();

  return (
    <Shell title="Budget" subtitle="Driver-based planning and budget-versus-actual variance analysis.">
      <Tabs defaultValue="line-items">
        <TabsList>
          <TabsTrigger value="drivers">Drivers</TabsTrigger>
          <TabsTrigger value="line-items">Line items</TabsTrigger>
          <TabsTrigger value="variance">Variance</TabsTrigger>
        </TabsList>
        <TabsContent value="drivers">
          <DriversPanel state={driversState} />
        </TabsContent>
        <TabsContent value="line-items">
          <LineItemsPanel state={driversState} />
        </TabsContent>
        <TabsContent value="variance">
          <VariancePanel />
        </TabsContent>
      </Tabs>
    </Shell>
  );
}
