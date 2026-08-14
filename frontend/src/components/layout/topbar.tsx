import { useQuery } from "@tanstack/react-query";
import { healthApi } from "@/api/health";
import { cn } from "@/lib/utils";

export function Topbar({ title, subtitle }: { title: string; subtitle?: string }) {
  const { data, isError } = useQuery({
    queryKey: ["health"],
    queryFn: healthApi.check,
    refetchInterval: 15_000,
    retry: false,
  });

  const online = !isError && data?.status === "ok";

  return (
    <header className="flex h-16 shrink-0 items-center justify-between border-b border-void-800 bg-void-950/60 px-8 backdrop-blur">
      <div>
        <h1 className="text-lg font-semibold text-void-50">{title}</h1>
        {subtitle && <p className="text-xs text-void-400">{subtitle}</p>}
      </div>
      <div className="flex items-center gap-2 rounded-full border border-void-700 bg-void-900/60 px-3 py-1.5 text-xs">
        <span
          className={cn(
            "h-1.5 w-1.5 rounded-full",
            online ? "bg-favorable-500 shadow-[0_0_8px_rgba(52,211,153,0.8)]" : "bg-unfavorable-500",
          )}
        />
        <span className={cn(online ? "text-void-300" : "text-unfavorable-500")}>
          {online ? "API online" : "API unreachable"}
        </span>
      </div>
    </header>
  );
}
