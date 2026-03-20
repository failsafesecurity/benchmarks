import { Link, useLocation } from "react-router-dom";
import type { ReactNode } from "react";

const NAV_ITEMS = [
  { path: "/", label: "Leaderboard" },
  { path: "/compare", label: "Compare" },
  { path: "/about", label: "About" },
];

export default function Layout({ children }: { children: ReactNode }) {
  const location = useLocation();

  return (
    <div className="min-h-screen flex flex-col">
      <header className="border-b border-gray-800 bg-gray-900/80 backdrop-blur-sm sticky top-0 z-50">
        <nav className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-14 flex items-center gap-8">
          <Link to="/" className="flex items-center gap-2 font-bold text-lg">
            <span className="text-orange-400">Claw Bench</span>
          </Link>
          <div className="flex gap-1">
            {NAV_ITEMS.map((item) => {
              const active = location.pathname === item.path;
              return (
                <Link
                  key={item.path}
                  to={item.path}
                  className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                    active
                      ? "bg-gray-800 text-white"
                      : "text-gray-400 hover:text-gray-200 hover:bg-gray-800/50"
                  }`}
                >
                  {item.label}
                </Link>
              );
            })}
          </div>
          <div className="flex-1" />
          <a
            href="https://github.com/nearai/benchmarks"
            target="_blank"
            rel="noopener noreferrer"
            className="text-gray-400 hover:text-gray-200 text-sm"
          >
            GitHub
          </a>
        </nav>
      </header>
      <main className="flex-1 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8 py-6">
        {children}
      </main>
      <footer className="border-t border-gray-800 py-4 text-center text-gray-500 text-xs">
        Claw Bench — Multi-framework AI agent benchmarks
      </footer>
    </div>
  );
}
