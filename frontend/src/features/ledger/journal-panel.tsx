import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2 } from "lucide-react";
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
import { formatCurrency, pascalToSnakeCase, shortId, snakeToPascalCase, titleCase } from "@/lib/format";
import { FORMULAS } from "@/features/calculator/formula-registry";
import type { JournalLine } from "@/api/types";

interface DraftLine {
  account: string;
  debit: string;
  credit: string;
  causalFormula: string;
}

function emptyLine(): DraftLine {
  return { account: "", debit: "", credit: "", causalFormula: "" };
}

export function JournalPanel() {
  const queryClient = useQueryClient();
  const entriesQuery = useQuery({ queryKey: ["journal-entries"], queryFn: ledgerApi.listEntries });

  const [description, setDescription] = useState("");
  const [date, setDate] = useState(() => new Date().toISOString().slice(0, 10));
  const [lines, setLines] = useState<DraftLine[]>([emptyLine(), emptyLine()]);
  const [causalParent, setCausalParent] = useState<string | null>(null);

  const postMutation = useMutation({
    mutationFn: () => {
      const [year, month] = date.split("-").map(Number);
      const journalLines: JournalLine[] = lines.map((l) => ({
        account: Number(l.account),
        debit: l.debit || "0",
        credit: l.credit || "0",
        causal_formula: l.causalFormula ? snakeToPascalCase(l.causalFormula) : null,
      }));
      return ledgerApi.postEntry({
        date,
        description,
        lines: journalLines,
        causal_parent: causalParent,
        source_document: "ManualEntry",
        period: { year, month },
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["journal-entries"] });
      queryClient.invalidateQueries({ queryKey: ["trial-balance"] });
      setDescription("");
      setLines([emptyLine(), emptyLine()]);
      setCausalParent(null);
    },
  });

  const updateLine = (index: number, patch: Partial<DraftLine>) => {
    setLines((prev) => prev.map((l, i) => (i === index ? { ...l, ...patch } : l)));
  };

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[26rem_1fr]">
      <Card>
        <CardHeader>
          <CardTitle>Post journal entry</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-1">
              <Label htmlFor="je-date">Date</Label>
              <Input id="je-date" type="date" value={date} onChange={(e) => setDate(e.target.value)} />
            </div>
            <div className="space-y-1">
              <Label htmlFor="je-desc">Description</Label>
              <Input
                id="je-desc"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
              />
            </div>
          </div>

          <div className="space-y-2">
            {lines.map((line, i) => (
              <div key={i} className="space-y-1.5 rounded-lg border border-void-800 p-2">
                <div className="grid grid-cols-[1fr_1fr_1fr_auto] gap-2">
                  <Input
                    placeholder="Account"
                    type="number"
                    value={line.account}
                    onChange={(e) => updateLine(i, { account: e.target.value })}
                  />
                  <Input
                    placeholder="Debit"
                    value={line.debit}
                    onChange={(e) => updateLine(i, { debit: e.target.value, credit: "" })}
                  />
                  <Input
                    placeholder="Credit"
                    value={line.credit}
                    onChange={(e) => updateLine(i, { credit: e.target.value, debit: "" })}
                  />
                  <Button
                    variant="ghost"
                    size="icon"
                    disabled={lines.length <= 2}
                    onClick={() => setLines((prev) => prev.filter((_, idx) => idx !== i))}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
                <Select
                  value={line.causalFormula || "__none__"}
                  onValueChange={(v) => updateLine(i, { causalFormula: v === "__none__" ? "" : v })}
                >
                  <SelectTrigger className="h-8 text-xs">
                    <SelectValue placeholder="Causal formula (optional)" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__none__">No causal formula</SelectItem>
                    {FORMULAS.map((f) => (
                      <SelectItem key={f.name} value={f.name}>
                        {f.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            ))}
            <Button variant="outline" size="sm" onClick={() => setLines((prev) => [...prev, emptyLine()])}>
              <Plus className="h-3.5 w-3.5" />
              Add line
            </Button>
          </div>

          <div className="space-y-1">
            <Label>Causal parent (optional)</Label>
            <Select
              value={causalParent ?? "__none__"}
              onValueChange={(v) => setCausalParent(v === "__none__" ? null : v)}
            >
              <SelectTrigger>
                <SelectValue placeholder="What caused this entry?" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__none__">This is an origin — no parent</SelectItem>
                {entriesQuery.data?.map((e) => (
                  <SelectItem key={e.id} value={e.id}>
                    {e.description} · {shortId(e.id)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {postMutation.isError && (
            <p className="text-xs text-unfavorable-500">{(postMutation.error as Error).message}</p>
          )}

          <Button
            className="w-full"
            disabled={!description || postMutation.isPending}
            onClick={() => postMutation.mutate()}
          >
            Post entry
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Entries</CardTitle>
        </CardHeader>
        <CardContent>
          {entriesQuery.data?.length === 0 && (
            <p className="text-sm text-void-500">No entries posted yet.</p>
          )}
          <div className="space-y-3">
            {entriesQuery.data
              ?.slice()
              .reverse()
              .map((entry) => (
                <div key={entry.id} className="rounded-lg border border-void-800 p-3">
                  <div className="mb-2 flex items-center justify-between">
                    <span className="text-sm font-medium text-void-100">{entry.description}</span>
                    <span className="font-mono text-xs text-void-500">{entry.date}</span>
                  </div>
                  {entry.causal_parent && (
                    <div className="mb-2 text-[11px] text-void-500">
                      ← caused by {shortId(entry.causal_parent)}
                    </div>
                  )}
                  <div className="space-y-1">
                    {entry.lines.map((line, i) => (
                      <div key={i} className="flex items-center justify-between gap-2 font-mono text-xs text-void-400">
                        <span>Account {line.account}</span>
                        <div className="flex items-center gap-2">
                          {line.causal_formula && (
                            <Badge variant="default">
                              {titleCase(pascalToSnakeCase(line.causal_formula))}
                            </Badge>
                          )}
                          <span>
                            {Number(line.debit) > 0
                              ? `Dr ${formatCurrency(line.debit)}`
                              : `Cr ${formatCurrency(line.credit)}`}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
