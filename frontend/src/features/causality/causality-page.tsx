import { Shell } from "@/components/layout/shell";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { FormulaGraphPanel } from "./formula-graph-panel";
import { JournalLineagePanel } from "./journal-lineage-panel";

export function CausalityPage() {
  return (
    <Shell
      title="Causality"
      subtitle="Trace any computed number back to the formulas and journal entries that produced it."
    >
      <Tabs defaultValue="formulas">
        <TabsList>
          <TabsTrigger value="formulas">Formula graph</TabsTrigger>
          <TabsTrigger value="journal">Journal lineage</TabsTrigger>
        </TabsList>
        <TabsContent value="formulas">
          <FormulaGraphPanel />
        </TabsContent>
        <TabsContent value="journal">
          <JournalLineagePanel />
        </TabsContent>
      </Tabs>
    </Shell>
  );
}
