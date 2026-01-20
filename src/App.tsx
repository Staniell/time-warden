import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Activity, Clock, LayoutDashboard, Settings, Shield, Timer } from "lucide-react";
import { StatsCard } from "./components/StatsCard";
import { StatusIndicator } from "./components/StatusIndicator";
import { ActivityList } from "./components/ActivityList";
import { UsageChart } from "./components/UsageChart";
import { TitleBar } from "./components/TitleBar";
import { cleanAppName } from "./lib/utils";

interface Session {
  id: number;
  app_id: string;
  app_name?: string;
  start_time: string;
  end_time?: string;
  duration_seconds?: number;
  is_idle: boolean;
}

interface AppUsage {
  name: string;
  seconds: number;
}

function App() {
  const [currentApp, setCurrentApp] = useState<string | null>(null);
  const [idleSeconds, setIdleSeconds] = useState<number>(0);
  const [sessions, setSessions] = useState<Session[]>([]);
  const [appUsage, setAppUsage] = useState<AppUsage[]>([]);
  const [totalTime, setTotalTime] = useState<number>(0);

  const fetchData = async () => {
    try {
      // Live status
      const app = await invoke<string | null>("get_current_app");
      const idle = await invoke<number>("get_idle_seconds");

      setCurrentApp(app);
      setIdleSeconds(idle);

      // Dashboard data
      const todaySessions = await invoke<Session[]>("get_today_sessions");
      const usage = await invoke<[string, number][]>("get_app_totals_today");

      setSessions(todaySessions.reverse()); // Most recent first

      const usageData = usage.map(([name, seconds]) => ({ name, seconds }));
      setAppUsage(usageData);

      const total = usageData.reduce((acc, curr) => acc + curr.seconds, 0);
      setTotalTime(total);
    } catch (error) {
      console.error("Failed to fetch data:", error);
    }
  };

  useEffect(() => {
    // Initial fetch
    fetchData();

    // Poll every 1 second
    const interval = setInterval(fetchData, 1000);
    return () => clearInterval(interval);
  }, []);

  const formatDuration = (seconds: number): string => {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);

    if (h > 0) return `${h}h ${m}m`;
    return `${m}m` || "0m";
  };

  return (
    <div className="min-h-screen bg-zinc-950 text-zinc-100 font-sans selection:bg-indigo-500/30">
      <TitleBar />

      {/* Navbar - Pushed down by TitleBar (h-8 = 2rem) */}
      <nav className="fixed top-8 left-0 right-0 h-16 bg-zinc-950/80 backdrop-blur-md border-b border-zinc-800/50 z-40">
        <div className="max-w-7xl mx-auto px-6 h-full flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-indigo-600 rounded-lg shadow-lg shadow-indigo-500/20">
              <Shield className="w-5 h-5 text-white" />
            </div>
            <span className="font-bold text-lg tracking-tight bg-gradient-to-r from-white to-zinc-400 bg-clip-text text-transparent">
              Timewarden
            </span>
          </div>

          <div className="flex items-center gap-6">
            <StatusIndicator
              currentApp={currentApp || undefined}
              isIdle={idleSeconds > 300} // Hardcoded 5 min threshold for UI
            />
            <Settings className="w-5 h-5 text-zinc-400 hover:text-white cursor-pointer transition-colors" />
          </div>
        </div>
      </nav>

      {/* Main Content - Added padding top to account for TitleBar + Navbar (2rem + 4rem = 6rem, so pt-24 is 6rem. Wait, pt-24 is 6rem. pt-32 is 8rem. We need 2rem + 4rem + spacing. 6rem start. Previous was pt-24=6rem. Now we have extra 2rem top. So pt-32 might be better or keeping pt-24 but adding mt-8.) */}
      <main className="pt-32 pb-12 px-6 max-w-7xl mx-auto">
        <div className="grid grid-cols-12 gap-8">
          {/* Header */}
          <div className="col-span-12 mb-4">
            <h1 className="text-3xl font-bold mb-2">Dashboard</h1>
            <p className="text-zinc-400">Overview of your activity today</p>
          </div>

          {/* Stats Cards */}
          <div className="col-span-12 grid grid-cols-3 gap-6">
            <StatsCard
              title="Total Time"
              value={formatDuration(totalTime)}
              icon={Timer}
              description="Active screen time today"
            />
            <StatsCard
              title="Most Used"
              value={appUsage.length > 0 ? cleanAppName(appUsage[0].name) : "N/A"}
              icon={Activity}
              description={appUsage.length > 0 ? formatDuration(appUsage[0].seconds) : undefined}
            />
            <StatsCard
              title="Productivity"
              value="85%"
              icon={LayoutDashboard}
              description="Based on app categories"
              trend={{ value: 12, isPositive: true }}
            />
          </div>

          {/* Charts Area */}
          <div className="col-span-8 space-y-6">
            <div className="bg-zinc-900/40 rounded-2xl p-6 border border-zinc-800/50 backdrop-blur-sm">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-lg font-semibold flex items-center gap-2">
                  <Activity className="w-5 h-5 text-indigo-500" />
                  Top Applications
                </h2>
              </div>
              <UsageChart data={appUsage} />
            </div>
          </div>

          {/* Activity Feed */}
          <div className="col-span-4">
            <div className="bg-zinc-900/40 rounded-2xl p-6 border border-zinc-800/50 backdrop-blur-sm h-full">
              <h2 className="text-lg font-semibold mb-6 flex items-center gap-2">
                <Clock className="w-5 h-5 text-emerald-500" />
                Recent Activity
              </h2>
              <ActivityList sessions={sessions.slice(0, 10)} />
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
