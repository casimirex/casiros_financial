import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { AlertTriangle } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { ledgerApi } from "@/api/ledger";
import type { JournalEntry } from "@/api/types";
import { formatCurrency, pascalToSnakeCase, shortId, titleCase } from "@/lib/format";

interface Lineage {
  chain: JournalEntry[];
  brokenParent: string | null;
}

// Walks causal_parent backward from `entry` through `byId`, root-first.
// causal_parent is *not* validated against existing entries when an entry is
// posted (see crates/erp/src/ledger/journal.rs — JournalEntry::new accepts
// any Uuid), so a parent id absent from the ledger is reported rather than
// silently treated as "no parent".
function buildLineage(entry: JournalEntry, byId: Map<string, JournalEntry>): Lineage {
  const chain: JournalEntry[] = [entry];
  let current = entry;
  while (current.causal_parent) {
    const parent = byId.get(current.causal_parent);
    if (!parent) {
      return { chain, brokenParent: current.causal_parent };
    }
    chain.unshift(parent);
    current = parent;
  }
  return { chain, brokenParent: null };
}

export function JournalLineagePanel() {
  const entriesQuery = useQuery({ queryKey: ["journal-entries"], queryFn: ledgerApi.listEntries });
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const entries = entriesQuery.data ?? [];
  const byId = new Map(entries.map((e) => [e.id, e]));
  const selected = selectedId ? byId.get(selectedId) : undefined;
  const lineage = selected ? buildLineage(selected, byId) : null;

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Pick a journal entry</CardTitle>
          <CardDescription>
            Every posted entry can declare a <code className="rounded bg-void-800 px-1 py-0.5 text-xs">causal_parent</code>{" "}
            (what caused it) and each line a <code className="rounded bg-void-800 px-1 py-0.5 text-xs">causal_formula</code>{" "}
            (what computed it) — set them when posting on the Ledger page to see a real chain here.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {entries.length === 0 ? (
            <p className="text-sm text-void-500">No journal entries posted yet.</p>
          ) : (
            <div className="max-w-md space-y-1">
              <Label>Entry</Label>
              <Select value={selectedId ?? undefined} onValueChange={setSelectedId}>
                <SelectTrigger>
                  <SelectValue placeholder="Select an entry" />
                </SelectTrigger>
                <SelectContent>
                  {entries
                    .slice()
                    .reverse()
                    .map((e) => (
                      <SelectItem key={e.id} value={e.id}>
                        {e.description} · {e.date} · {shortId(e.id)}
                      </SelectItem>
                    ))}
                </SelectContent>
              </Select>
            </div>
          )}
        </CardContent>
      </Card>

      {lineage && (
        <Card>
          <CardHeader>
            <CardTitle>Causal chain</CardTitle>
            <CardDescription>
              {lineage.chain.length === 1
                ? "This entry has no causal parent — it is its own origin."
                : `${lineage.chain.length} entries, origin first.`}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-0">
              {lineage.chain.map((entry, i) => (
                <div key={entry.id} className="relative pb-6 pl-6 last:pb-0">
                  {i < lineage.chain.length - 1 && (
                    <div className="absolute top-3 left-[5px] h-full w-px bg-void-700" />
                  )}
                  <div
                    className={`absolute top-1.5 left-0 h-2.5 w-2.5 rounded-full ${
                      entry.id === selectedId ? "bg-signal-400" : "bg-void-600"
                    }`}
                  />
                  <div
                    className={`rounded-lg border p-3 ${
                      entry.id === selectedId
                        ? "border-signal-500/50 bg-signal-500/10"
                        : "border-void-800 bg-void-900/60"
                    }`}
                  >
                    <div className="mb-2 flex items-center justify-between">
                      <span className="text-sm font-medium text-void-100">{entry.description}</span>
                      <span className="font-mono text-xs text-void-500">{entry.date}</span>
                    </div>
                    <div className="space-y-1">
                      {entry.lines.map((line, li) => (
                        <div key={li} className="flex items-center justify-between gap-2 font-mono text-xs">
                          <span className="text-void-400">Account {line.account}</span>
                          <div className="flex items-center gap-2">
                            {line.causal_formula && (
                              <Badge variant="default">
                                {titleCase(pascalToSnakeCase(line.causal_formula))}
                              </Badge>
                            )}
                            <span className="text-void-300">
                              {Number(line.debit) > 0
                                ? `Dr ${formatCurrency(line.debit)}`
                                : `Cr ${formatCurrency(line.credit)}`}
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              ))}
              {lineage.brokenParent && (
                <div className="flex items-center gap-2 rounded-lg border border-caution-500/30 bg-caution-500/10 p-3 text-xs text-caution-500">
                  <AlertTriangle className="h-4 w-4 shrink-0" />
                  Declares a causal parent ({shortId(lineage.brokenParent)}) not found in the
                  ledger — the chain stops here.
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
