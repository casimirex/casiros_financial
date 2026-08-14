import { Shell } from "@/components/layout/shell";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { SuppliersPanel } from "./suppliers-panel";
import { InvoicesPanel } from "./invoices-panel";
import { AgingPanel } from "./aging-panel";
import { PaymentsPanel } from "./payments-panel";

export function ApPage() {
  return (
    <Shell title="Accounts Payable" subtitle="Suppliers, invoices, aging, and payment proposals.">
      <Tabs defaultValue="invoices">
        <TabsList>
          <TabsTrigger value="suppliers">Suppliers</TabsTrigger>
          <TabsTrigger value="invoices">Invoices</TabsTrigger>
          <TabsTrigger value="aging">Aging</TabsTrigger>
          <TabsTrigger value="payments">Payment proposals</TabsTrigger>
        </TabsList>
        <TabsContent value="suppliers">
          <SuppliersPanel />
        </TabsContent>
        <TabsContent value="invoices">
          <InvoicesPanel />
        </TabsContent>
        <TabsContent value="aging">
          <AgingPanel />
        </TabsContent>
        <TabsContent value="payments">
          <PaymentsPanel />
        </TabsContent>
      </Tabs>
    </Shell>
  );
}
