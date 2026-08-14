import type { TaxBracket } from "@/api/types";

// Local, percent-and-string-backed mirror of TaxBracket[] so the form can be
// edited freely before being parsed into the wire shape on submit. The last
// bracket is always unbounded, matching TaxJurisdiction::new's invariant
// (see crates/erp/src/tax/jurisdiction.rs).
export interface BracketForm {
  upperBound: string;
  ratePercent: string;
}

export const defaultBrackets: BracketForm[] = [
  { upperBound: "50000", ratePercent: "10" },
  { upperBound: "", ratePercent: "24" },
];

export function bracketsFromForm(brackets: BracketForm[]): TaxBracket[] {
  return brackets.map((b, i) => ({
    upper_bound: i === brackets.length - 1 ? null : b.upperBound,
    rate: (Number(b.ratePercent) / 100).toString(),
  }));
}
