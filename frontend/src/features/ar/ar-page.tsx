import { Shell } from "@/components/layout/shell";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { CustomersPanel } from "./customers-panel";
import { InvoicesPanel } from "./invoices-panel";
import { ReceiptsPanel } from "./receipts-panel";

export function ArPage() {
  return (
    <Shell title="Accounts Receivable" subtitle="Customers, invoices, and cash receipt allocation.">
      <Tabs defaultValue="invoices">
        <TabsList>
          <TabsTrigger value="customers">Customers</TabsTrigger>
          <TabsTrigger value="invoices">Invoices</TabsTrigger>
          <TabsTrigger value="receipts">Receipts</TabsTrigger>
        </TabsList>
        <TabsContent value="customers">
          <CustomersPanel />
        </TabsContent>
        <TabsContent value="invoices">
          <InvoicesPanel />
        </TabsContent>
        <TabsContent value="receipts">
          <ReceiptsPanel />
        </TabsContent>
      </Tabs>
    </Shell>
  );
}
