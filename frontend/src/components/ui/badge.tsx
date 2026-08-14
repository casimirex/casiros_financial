import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium",
  {
    variants: {
      variant: {
        default: "border-signal-500/30 bg-signal-500/10 text-signal-400",
        favorable: "border-favorable-500/30 bg-favorable-500/10 text-favorable-500",
        unfavorable: "border-unfavorable-500/30 bg-unfavorable-500/10 text-unfavorable-500",
        caution: "border-caution-500/30 bg-caution-500/10 text-caution-500",
        neutral: "border-void-600 bg-void-800 text-void-300",
        nova: "border-nova-500/30 bg-nova-500/10 text-nova-400",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLSpanElement>,
    VariantProps<typeof badgeVariants> {}

export function Badge({ className, variant, ...props }: BadgeProps) {
  return <span className={cn(badgeVariants({ variant }), className)} {...props} />;
}
