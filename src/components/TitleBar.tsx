import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";

export function TitleBar() {
  const appWindow = getCurrentWindow();

  return (
    <div className="fixed top-0 left-0 right-0 h-8 bg-[#0f1117] flex items-center justify-between px-2 select-none z-50 border-b border-zinc-900/50">
      {/* Drag Region - takes up all available space to the left of controls */}
      <div className="flex-1 h-full" data-tauri-drag-region />

      {/* Window Controls - z-index ensures they sit above if needed, but flex separation is safer */}
      <div className="flex items-center gap-1">
        <button
          onClick={() => appWindow.minimize()}
          className="p-1.5 rounded-md hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
        >
          <Minus className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={() => appWindow.toggleMaximize()}
          className="p-1.5 rounded-md hover:bg-zinc-800 text-zinc-400 hover:text-white transition-colors"
        >
          <Square className="w-3.5 h-3.5" />
        </button>
        <button
          onClick={() => appWindow.close()}
          className="p-1.5 rounded-md hover:bg-rose-500/10 text-zinc-400 hover:text-rose-500 transition-colors"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  );
}
