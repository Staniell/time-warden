import { LucideIcon } from "lucide-react";
import { cn } from "../lib/utils";

interface StatsCardProps {
  title: string;
  value: string | number;
  description?: string;
  icon: LucideIcon;
  trend?: {
    value: number;
    isPositive: boolean;
  };
  className?: string;
}

export function StatsCard({ title, value, description, icon: Icon, trend, className }: StatsCardProps) {
  return (
    <div
      className={cn(
        "p-5 rounded-xl bg-zinc-900/50 border border-zinc-800/50 hover:border-zinc-700/50 transition-colors backdrop-blur-sm",
        className,
      )}
    >
      <div className="flex justify-between items-start mb-2">
        <div className="p-2 rounded-lg bg-indigo-500/10 text-indigo-400">
          <Icon className="w-5 h-5" />
        </div>
        {trend && (
          <div
            className={cn(
              "text-xs font-medium px-2 py-1 rounded-full",
              trend.isPositive ? "bg-emerald-500/10 text-emerald-500" : "bg-rose-500/10 text-rose-500",
            )}
          >
            {trend.isPositive ? "+" : ""}
            {trend.value}%
          </div>
        )}
      </div>
      <div>
        <h3 className="text-sm font-medium text-zinc-400 mb-1">{title}</h3>
        <div className="text-2xl font-bold text-zinc-100 tracking-tight">{value}</div>
        {description && <p className="text-xs text-zinc-500 mt-1">{description}</p>}
      </div>
    </div>
  );
}
