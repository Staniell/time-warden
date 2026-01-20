import { Bar, BarChart, ResponsiveContainer, XAxis, YAxis, Tooltip, Cell } from "recharts";
import { cn, cleanAppName } from "../lib/utils";

interface AppUsage {
  name: string;
  seconds: number;
}

interface UsageChartProps {
  data: AppUsage[];
  className?: string;
}

export function UsageChart({ data, className }: UsageChartProps) {
  // Take top 5 apps
  const chartData = data.slice(0, 5).map((item) => ({
    ...item,
    name: cleanAppName(item.name),
    minutes: Math.round(item.seconds / 60),
  }));

  if (chartData.length === 0) {
    return (
      <div
        className={cn(
          "flex flex-col items-center justify-center p-12 text-zinc-500 bg-zinc-900/30 rounded-xl border border-dashed border-zinc-800",
          className,
        )}
      >
        <p className="text-sm">Not enough data to display chart</p>
      </div>
    );
  }

  return (
    <div
      className={cn(
        "h-[300px] w-full bg-zinc-900/30 rounded-xl p-4 border border-zinc-800/50 overflow-hidden",
        className,
      )}
    >
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={chartData} layout="vertical" margin={{ top: 0, right: 30, left: 10, bottom: 0 }}>
          <XAxis type="number" hide />
          <YAxis
            dataKey="name"
            type="category"
            width={100}
            tick={{ fill: "#a1a1aa", fontSize: 12 }}
            axisLine={false}
            tickLine={false}
          />
          <Tooltip
            cursor={{ fill: "rgba(255,255,255,0.05)" }}
            contentStyle={{
              backgroundColor: "#18181b",
              borderColor: "#27272a",
              borderRadius: "8px",
              color: "#e4e4e7",
            }}
            itemStyle={{ color: "#a5b4fc" }}
            formatter={(value: number | undefined) => [`${value || 0} mins`, "Time"]}
          />
          <Bar dataKey="minutes" radius={[0, 4, 4, 0]}>
            {chartData.map((_, index) => (
              <Cell key={`cell-${index}`} fill={index === 0 ? "#6366f1" : "#3f3f46"} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
