import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { PaymentTermsForm } from "./payment-terms";

export function PaymentTermsFields({
  idPrefix,
  value,
  onChange,
}: {
  idPrefix: string;
  value: PaymentTermsForm;
  onChange: (value: PaymentTermsForm) => void;
}) {
  return (
    <div className="grid grid-cols-3 gap-2">
      <div className="space-y-1">
        <Label htmlFor={`${idPrefix}-net-days`}>Net days</Label>
        <Input
          id={`${idPrefix}-net-days`}
          type="number"
          min="0"
          value={value.netDays}
          onChange={(e) => onChange({ ...value, netDays: e.target.value })}
        />
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${idPrefix}-discount-pct`}>Discount %</Label>
        <Input
          id={`${idPrefix}-discount-pct`}
          type="number"
          min="0"
          placeholder="none"
          value={value.discountPercent}
          onChange={(e) => onChange({ ...value, discountPercent: e.target.value })}
        />
      </div>
      <div className="space-y-1">
        <Label htmlFor={`${idPrefix}-discount-days`}>Discount days</Label>
        <Input
          id={`${idPrefix}-discount-days`}
          type="number"
          min="0"
          placeholder="none"
          value={value.discountDays}
          onChange={(e) => onChange({ ...value, discountDays: e.target.value })}
        />
      </div>
    </div>
  );
}
