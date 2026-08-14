import { NavLink } from "react-router-dom";
import {
  Orbit,
  Calculator,
  ScrollText,
  BookOpen,
  Gauge,
  CreditCard,
  Wallet,
  Landmark,
  Percent,
  PieChart,
  Waypoints,
} from "lucide-react";
import { cn } from "@/lib/utils";

const NAV_ITEMS = [
  { to: "/", label: "Overview", icon: Gauge, end: true },
  { to: "/multiverse", label: "The Multiverse", icon: Orbit, end: false },
  { to: "/calculator", label: "Calculator", icon: Calculator, end: false },
  { to: "/narrative", label: "Narrative", icon: ScrollText, end: false },
  { to: "/ledger", label: "Ledger", icon: BookOpen, end: false },
  { to: "/ap", label: "Payable", icon: CreditCard, end: false },
  { to: "/ar", label: "Receivable", icon: Wallet, end: false },
  { to: "/treasury", label: "Treasury", icon: Landmark, end: false },
  { to: "/tax", label: "Tax", icon: Percent, end: false },
  { to: "/budget", label: "Budget", icon: PieChart, end: false },
  { to: "/causality", label: "Causality", icon: Waypoints, end: false },
] as const;

export function Sidebar() {
  return (
    <aside className="flex w-64 shrink-0 flex-col border-r border-void-800 bg-void-950/80">
      <div className="flex h-16 items-center gap-3 px-6">
        <div className="relative h-8 w-8 shrink-0">
          <div className="absolute inset-0 animate-pulse-slow rounded-full bg-signal-500/30 blur-md" />
          <div className="relative flex h-8 w-8 items-center justify-center rounded-full border border-signal-500/50 bg-void-900">
            <Orbit className="h-4 w-4 text-signal-400" />
          </div>
        </div>
        <div>
          <div className="text-sm font-bold tracking-wide text-void-50">CASIROS</div>
          <div className="text-[10px] uppercase tracking-widest text-void-500">Mission Control</div>
        </div>
      </div>

      <nav className="flex flex-1 flex-col gap-1 px-3 py-4">
        {NAV_ITEMS.map(({ to, label, icon: Icon, end }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) =>
              cn(
                "group flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
                isActive
                  ? "bg-signal-500/10 text-signal-400"
                  : "text-void-400 hover:bg-void-800/60 hover:text-void-100",
              )
            }
          >
            <Icon className="h-4 w-4" />
            {label}
          </NavLink>
        ))}
      </nav>

      <div className="border-t border-void-800 px-6 py-4 text-[11px] leading-relaxed text-void-500">
        Every number here traces to a formula and a scenario.
        <br />
        Nothing is hand-waved.
      </div>
    </aside>
  );
}
