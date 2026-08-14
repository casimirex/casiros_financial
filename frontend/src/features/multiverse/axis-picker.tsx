import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { METRIC_KEYS, type MetricKey } from "@/api/types";
import { METRIC_LABELS } from "./metric-labels";

function AxisSelect({
  label,
  value,
  onChange,
}: {
  label: string;
  value: MetricKey;
  onChange: (value: MetricKey) => void;
}) {
  return (
    <div className="space-y-1">
      <Label>{label}</Label>
      <Select value={value} onValueChange={(v) => onChange(v as MetricKey)}>
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {METRIC_KEYS.map((key) => (
            <SelectItem key={key} value={key}>
              {METRIC_LABELS[key]}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

export interface AxisSelection {
  x: MetricKey;
  y: MetricKey;
  z: MetricKey;
  color: MetricKey;
}

export function AxisPicker({
  selection,
  onChange,
}: {
  selection: AxisSelection;
  onChange: (selection: AxisSelection) => void;
}) {
  return (
    <div className="grid grid-cols-2 gap-3">
      <AxisSelect label="X axis" value={selection.x} onChange={(x) => onChange({ ...selection, x })} />
      <AxisSelect label="Y axis" value={selection.y} onChange={(y) => onChange({ ...selection, y })} />
      <AxisSelect label="Z axis" value={selection.z} onChange={(z) => onChange({ ...selection, z })} />
      <AxisSelect
        label="Color (favorability)"
        value={selection.color}
        onChange={(color) => onChange({ ...selection, color })}
      />
    </div>
  );
}
