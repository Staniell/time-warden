import { Activity, Pause } from "lucide-react";
import { cn, cleanAppName } from "../lib/utils";

interface StatusIndicatorProps {
  currentApp?: string;
  isIdle?: boolean;
  className?: string;
}

export function StatusIndicator({ currentApp, isIdle, className }: StatusIndicatorProps) {
  if (!currentApp) {
    return (
      <div
        className={cn(
          "flex items-center gap-2 px-3 py-1.5 rounded-full bg-zinc-800/50 text-zinc-400 border border-zinc-700/50 backdrop-blur-sm",
          className,
        )}
      >
        <div className="w-2 h-2 rounded-full bg-zinc-500" />
        <span className="text-xs font-medium">Inactive</span>
      </div>
    );
  }

  if (isIdle) {
    return (
      <div
        className={cn(
          "flex items-center gap-2 px-3 py-1.5 rounded-full bg-amber-500/10 text-amber-500 border border-amber-500/20 backdrop-blur-sm",
          className,
        )}
      >
        <Pause className="w-3.5 h-3.5" />
        <span className="text-xs font-medium">Idle</span>
      </div>
    );
  }

  return (
    <div
      className={cn(
        "flex items-center gap-2 px-3 py-1.5 rounded-full bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 backdrop-blur-sm",
        className,
      )}
    >
      <Activity className="w-3.5 h-3.5 animate-pulse" />
      <span className="text-xs font-medium">
        Tracking: <span className="font-bold">{cleanAppName(currentApp)}</span>
      </span>
    </div>
  );
}
