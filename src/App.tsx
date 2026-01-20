import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <div className="min-h-screen bg-[#0f1117] text-slate-200 font-sans selection:bg-indigo-500/30">
      <nav className="border-b border-slate-800/60 bg-[#0f1117]/80 backdrop-blur-md sticky top-0 z-50">
        <div className="max-w-5xl mx-auto px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-indigo-600 flex items-center justify-center shadow-lg shadow-indigo-500/20">
              <span className="text-white font-bold">W</span>
            </div>
            <span className="font-semibold text-lg tracking-tight">Timewarden</span>
          </div>
          <div className="flex gap-6 text-sm font-medium text-slate-400">
            <span className="text-indigo-400 cursor-default">Dashboard</span>
            <span className="hover:text-slate-200 transition-colors cursor-pointer">Schedules</span>
            <span className="hover:text-slate-200 transition-colors cursor-pointer">Reports</span>
          </div>
        </div>
      </nav>

      <main className="max-w-5xl mx-auto px-6 py-12">
        <div className="relative group mb-12">
          <div className="absolute -inset-1 bg-gradient-to-r from-indigo-500 to-purple-600 rounded-2xl blur opacity-15 group-hover:opacity-25 transition duration-1000 group-hover:duration-200"></div>
          <div className="relative bg-[#161922] border border-slate-800/60 rounded-2xl p-8 flex flex-col md:flex-row items-center justify-between gap-8">
            <div className="space-y-4">
              <h1 className="text-4xl font-bold text-white tracking-tight">
                Guardian of your{" "}
                <span className="text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-purple-400">
                  focus.
                </span>
              </h1>
              <p className="text-slate-400 max-w-md leading-relaxed">
                Time Warden is initialized and ready to track your digital sessions with zero overhead.
              </p>
            </div>

            <form
              className="flex flex-col gap-3 w-full max-w-sm"
              onSubmit={(e) => {
                e.preventDefault();
                greet();
              }}
            >
              <div className="relative">
                <input
                  id="greet-input"
                  className="w-full bg-[#1c212c] border border-slate-700/50 rounded-xl px-4 py-3 text-slate-100 placeholder:text-slate-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-all"
                  onChange={(e) => setName(e.currentTarget.value)}
                  placeholder="App Status Check..."
                />
              </div>
              <button
                type="submit"
                className="bg-indigo-600 hover:bg-indigo-500 text-white font-semibold py-3 px-6 rounded-xl shadow-lg shadow-indigo-600/20 active:scale-[0.98] transition-all"
              >
                Verify Core Bridge
              </button>
            </form>
          </div>
        </div>

        {greetMsg && (
          <div className="animate-in fade-in slide-in-from-top-4 duration-500">
            <div className="bg-indigo-500/10 border border-indigo-500/20 rounded-xl p-4 flex items-center gap-3 text-indigo-300">
              <div className="w-2 h-2 rounded-full bg-indigo-500 animate-pulse"></div>
              {greetMsg}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
