import { Link } from "react-router-dom";
import { Orbit, Calculator, ScrollText, BookOpen, ArrowUpRight } from "lucide-react";
import { Shell } from "@/components/layout/shell";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

const ENTRY_POINTS = [
  {
    to: "/multiverse",
    icon: Orbit,
    title: "The Multiverse",
    description:
      "Run a Monte Carlo simulation and watch every scenario take shape as a point in 3D space — turn the field to see the distribution from another angle.",
    accent: "text-signal-400",
  },
  {
    to: "/calculator",
    icon: Calculator,
    title: "Calculator",
    description: "Evaluate any of the 44 core formulas directly, from time-value-of-money to WACC.",
    accent: "text-nova-400",
  },
  {
    to: "/narrative",
    icon: ScrollText,
    title: "Narrative",
    description: "Turn a set of computed metrics into a CFO-style analysis memo, generated deterministically.",
    accent: "text-favorable-500",
  },
  {
    to: "/ledger",
    icon: BookOpen,
    title: "Ledger",
    description: "The causal general ledger — register accounts, post balanced entries, read the trial balance.",
    accent: "text-caution-500",
  },
] as const;

export function DashboardPage() {
  return (
    <Shell title="Overview" subtitle="A financial operating system where every number has a traceable origin.">
      <div className="mb-8">
        <p className="max-w-2xl text-sm leading-relaxed text-void-400">
          CASIROS is built on a simple discipline: no formula runs without a documented example, no
          journal entry posts without balancing, and no simulated outcome is presented without its
          full distribution. This is the control room for all of it.
        </p>
      </div>

      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 xl:grid-cols-4">
        {ENTRY_POINTS.map(({ to, icon: Icon, title, description, accent }) => (
          <Link key={to} to={to} className="group">
            <Card className="h-full transition-all duration-200 group-hover:-translate-y-1 group-hover:border-void-500 group-hover:shadow-2xl group-hover:shadow-signal-500/10">
              <CardHeader>
                <div className="mb-2 flex items-center justify-between">
                  <Icon className={`h-6 w-6 ${accent}`} />
                  <ArrowUpRight className="h-4 w-4 text-void-600 transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5 group-hover:text-void-300" />
                </div>
                <CardTitle>{title}</CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription>{description}</CardDescription>
              </CardContent>
            </Card>
          </Link>
        ))}
      </div>
    </Shell>
  );
}
