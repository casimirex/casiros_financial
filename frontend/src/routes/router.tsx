import { lazy, Suspense } from "react";
import { createBrowserRouter } from "react-router-dom";
import { DashboardPage } from "@/features/dashboard/dashboard-page";
import { CalculatorPage } from "@/features/calculator/calculator-page";
import { NarrativePage } from "@/features/narrative/narrative-page";
import { LedgerPage } from "@/features/ledger/ledger-page";
import { MultiverseFallback } from "./multiverse-fallback";

// The Multiverse view pulls in three.js + react-three-fiber, by far the
// heaviest dependency in the app — code-split so every other route's bundle
// stays lean and that weight loads only when a visitor actually opens it.
const MultiversePage = lazy(() =>
  import("@/features/multiverse/multiverse-page").then((m) => ({ default: m.MultiversePage })),
);

export const router = createBrowserRouter([
  { path: "/", element: <DashboardPage /> },
  {
    path: "/multiverse",
    element: (
      <Suspense fallback={<MultiverseFallback />}>
        <MultiversePage />
      </Suspense>
    ),
  },
  { path: "/calculator", element: <CalculatorPage /> },
  { path: "/narrative", element: <NarrativePage /> },
  { path: "/ledger", element: <LedgerPage /> },
]);
