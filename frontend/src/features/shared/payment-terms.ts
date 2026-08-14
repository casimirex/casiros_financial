import type { PaymentTerms } from "@/api/types";

// Local, string-backed mirror of PaymentTerms so empty optional fields can be
// typed through freely before being parsed into the wire shape on submit.
export interface PaymentTermsForm {
  netDays: string;
  discountPercent: string;
  discountDays: string;
}

export const emptyPaymentTermsForm: PaymentTermsForm = {
  netDays: "30",
  discountPercent: "",
  discountDays: "",
};

export function paymentTermsFromForm(form: PaymentTermsForm): PaymentTerms {
  return {
    net_days: Number(form.netDays) || 0,
    discount_percent: form.discountPercent ? (Number(form.discountPercent) / 100).toString() : null,
    discount_days: form.discountDays ? Number(form.discountDays) : null,
  };
}

export function formatPaymentTerms(terms: PaymentTerms): string {
  const parts = [`net ${terms.net_days}`];
  if (terms.discount_percent && terms.discount_days) {
    parts.push(`${(Number(terms.discount_percent) * 100).toFixed(0)}%/${terms.discount_days}`);
  }
  return parts.join(" · ");
}
