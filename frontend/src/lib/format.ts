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

// CurrencyCode crosses the wire as a raw [u8; 3] byte array (see
// crates/erp/src/treasury/fx.rs), not a 3-letter string.
export function currencyCodeToBytes(code: string): [number, number, number] {
  const padded = code.toUpperCase().padEnd(3, " ").slice(0, 3);
  return [padded.charCodeAt(0), padded.charCodeAt(1), padded.charCodeAt(2)];
}

export function bytesToCurrencyCode(bytes: [number, number, number]): string {
  return String.fromCharCode(...bytes).trim();
}

export function shortId(id: string): string {
  return id.slice(0, 8);
}

// FormulaNode crosses the wire as its bare Rust variant name (e.g.
// "DupontRoe" — see api/types.ts's FormulaNodeVariant comment), not the
// snake_case name FormulaNode::name()/from_name() use. These convert
// between the two so the frontend can build /causality/formulas/{name} URLs
// and human-readable labels from a PascalCase wire value.
export function pascalToSnakeCase(pascal: string): string {
  return pascal.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
}

export function snakeToPascalCase(snake: string): string {
  return snake
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join("");
}

export function titleCase(snake: string): string {
  return snake
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}
