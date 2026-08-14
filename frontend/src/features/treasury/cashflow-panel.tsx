import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Plus, TrendingUp } from "lucide-react";
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
import { treasuryApi } from "@/api/treasury";
import type { CashFlowCategory, CashFlowItem } from "@/api/types";
import { formatCurrency } from "@/lib/format";

const CATEGORIES: CashFlowCategory[] = ["Operating", "Investing", "Financing"];

export function CashflowPanel() {
  const [items, setItems] = useState<CashFlowItem[]>([]);
  const [category, setCategory] = useState<CashFlowCategory>("Operating");
  const [description, setDescription] = useState("");
  const [amount, setAmount] = useState("");
  const [date, setDate] = useState(new Date().toISOString().slice(0, 10));

  const addMutation = useMutation({
    mutationFn: (item: CashFlowItem) => treasuryApi.addCashflowItem(item),
    onSuccess: (item) => {
      setItems((prev) => [...prev, item]);
      setDescription("");
      setAmount("");
    },
  });

  const [openingBalance, setOpeningBalance] = useState("");
  const [asOf, setAsOf] = useState(new Date().toISOString().slice(0, 10));

  const forecastMutation = useMutation({
    mutationFn: async () => {
      const [projection, shortfall] = await Promise.all([
        treasuryApi.projection(openingBalance, asOf),
        treasuryApi.shortfall(openingBalance),
      ]);
      return { projection, shortfall };
    },
  });

  return (
    <div className="grid grid-cols-1 gap-6 lg:grid-cols-[22rem_1fr]">
      <div className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>Add forecast item</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="space-y-1">
              <Label>Category</Label>
              <Select value={category} onValueChange={(v) => setCategory(v as CashFlowCategory)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {CATEGORIES.map((c) => (
                    <SelectItem key={c} value={c}>
                      {c}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <Label htmlFor="cf-description">Description</Label>
              <Input
                id="cf-description"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
              />
            </div>
            <div className="grid grid-cols-2 gap-2">
              <div className="space-y-1">
                <Label htmlFor="cf-amount">Amount (± inflow)</Label>
                <Input
                  id="cf-amount"
                  type="number"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                />
              </div>
              <div className="space-y-1">
                <Label htmlFor="cf-date">Date</Label>
                <Input id="cf-date" type="date" value={date} onChange={(e) => setDate(e.target.value)} />
              </div>
            </div>
            <Button
              className="w-full"
              disabled={!description || !amount || addMutation.isPending}
              onClick={() => addMutation.mutate({ category, description, amount, date })}
            >
              <Plus className="h-4 w-4" />
              Add item
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Forecast</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="space-y-1">
              <Label htmlFor="cf-opening">Opening balance</Label>
              <Input
                id="cf-opening"
                type="number"
                value={openingBalance}
                onChange={(e) => setOpeningBalance(e.target.value)}
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="cf-as-of">Project to</Label>
              <Input id="cf-as-of" type="date" value={asOf} onChange={(e) => setAsOf(e.target.value)} />
            </div>
            <Button
              variant="secondary"
              className="w-full"
              disabled={!openingBalance || forecastMutation.isPending}
              onClick={() => forecastMutation.mutate()}
            >
              <TrendingUp className="h-4 w-4" />
              Project
            </Button>
            {forecastMutation.data && (
              <div className="space-y-2 border-t border-void-700 pt-3 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-void-500">Projected balance</span>
                  <span className="font-mono text-void-100">
                    {formatCurrency(forecastMutation.data.projection.balance)}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-void-500">First shortfall</span>
                  {forecastMutation.data.shortfall.shortfall_date ? (
                    <Badge variant="unfavorable">
                      {forecastMutation.data.shortfall.shortfall_date}
                    </Badge>
                  ) : (
                    <Badge variant="favorable">none projected</Badge>
                  )}
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Forecast items (this session)</CardTitle>
        </CardHeader>
        <CardContent>
          {items.length === 0 && (
            <p className="text-sm text-void-500">No items added yet in this session.</p>
          )}
          <div className="divide-y divide-void-800">
            {items.map((item, i) => (
              <div key={`${item.date}-${item.description}-${i}`} className="flex items-center justify-between py-3">
                <div>
                  <div className="text-sm text-void-100">{item.description}</div>
                  <div className="text-xs text-void-500">
                    {item.date} · {item.category}
                  </div>
                </div>
                <span
                  className={`font-mono text-sm ${
                    item.amount.startsWith("-") ? "text-unfavorable-500" : "text-favorable-500"
                  }`}
                >
                  {formatCurrency(item.amount)}
                </span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
