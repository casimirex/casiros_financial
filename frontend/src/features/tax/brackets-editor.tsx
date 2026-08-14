import { Plus, X } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import type { BracketForm } from "./brackets";

export function BracketsEditor({
  brackets,
  onChange,
}: {
  brackets: BracketForm[];
  onChange: (brackets: BracketForm[]) => void;
}) {
  const update = (index: number, patch: Partial<BracketForm>) => {
    onChange(brackets.map((b, i) => (i === index ? { ...b, ...patch } : b)));
  };

  return (
    <div className="space-y-2">
      <Label>Progressive brackets</Label>
      {brackets.map((bracket, i) => {
        const isLast = i === brackets.length - 1;
        return (
          <div key={i} className="flex items-center gap-2">
            <Input
              type="number"
              min="0"
              placeholder={isLast ? "unbounded" : "upper bound"}
              disabled={isLast}
              value={isLast ? "" : bracket.upperBound}
              onChange={(e) => update(i, { upperBound: e.target.value })}
              className="flex-1"
            />
            <Input
              type="number"
              min="0"
              max="100"
              placeholder="rate %"
              value={bracket.ratePercent}
              onChange={(e) => update(i, { ratePercent: e.target.value })}
              className="w-24"
            />
            <Button
              variant="ghost"
              size="icon"
              disabled={brackets.length <= 1}
              onClick={() => onChange(brackets.filter((_, idx) => idx !== i))}
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        );
      })}
      <Button
        variant="secondary"
        size="sm"
        onClick={() =>
          onChange([
            ...brackets.slice(0, -1),
            { upperBound: "", ratePercent: brackets.at(-1)?.ratePercent ?? "0" },
            brackets[brackets.length - 1],
          ])
        }
      >
        <Plus className="h-4 w-4" />
        Add bracket
      </Button>
    </div>
  );
}
