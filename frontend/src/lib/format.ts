// The API serializes every rust_decimal::Decimal as a JSON string (never a
// float — see crates/api/src/routes/calculate.rs), so every numeric value
// arrives here as a string and is parsed only for display, never fed back
// into a request without round-tripping through the original string.

export function formatDecimalString(value: string, fractionDigits = 2): string {
  const parsed = Number.parseFloat(value);
  if (Number.isNaN(parsed)) return value;
  return parsed.toLocaleString(undefined, {
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  });
}

export function formatCurrency(value: string, fractionDigits = 2): string {
  const parsed = Number.parseFloat(value);
  if (Number.isNaN(parsed)) return value;
  const sign = parsed < 0 ? "-" : "";
  return `${sign}$${Math.abs(parsed).toLocaleString(undefined, {
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  })}`;
}

export function formatPercent(value: string, fractionDigits = 1): string {
  const parsed = Number.parseFloat(value);
  if (Number.isNaN(parsed)) return value;
  return `${(parsed * 100).toFixed(fractionDigits)}%`;
}

export function toNumber(value: string): number {
  const parsed = Number.parseFloat(value);
  return Number.isNaN(parsed) ? 0 : parsed;
}
