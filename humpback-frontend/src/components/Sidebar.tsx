import { Code, Database, Plus } from "lucide-react";

// Components
export const Sidebar: React.FC<{
  activeView: string;
  setActiveView: (view: string) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
}> = ({ activeView, setActiveView, searchQuery, setSearchQuery }) => {
  return (
    <div className="w-92 bg-gray-800 text-white h-screen fixed left-0 top-0 flex flex-col">
      <div className="p-4">
        <h1 className="text-xl font-bold flex items-center gap-2">
          <span className="text-2xl">🐋</span>
          Humpback KV
        </h1>
      </div>

      <div className="p-4">
        <div className="relative">
          <input
            type="text"
            placeholder="Search keys..."
            className="w-full pl-10 pr-4 py-2 bg-gray-700 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
      </div>

      <nav className="flex-1 px-4">
        <button
          onClick={() => setActiveView("database")}
          className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg mb-2 transition-colors ${
            activeView === "database" ? "bg-blue-600" : "hover:bg-gray-700"
          }`}
        >
          <Database className="h-4 w-4" />
          Database Items
        </button>

        <button
          onClick={() => setActiveView("add")}
          className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg mb-2 transition-colors ${
            activeView === "add" ? "bg-blue-600" : "hover:bg-gray-700"
          }`}
        >
          <Plus className="h-4 w-4" />
          Add Item
        </button>

        <button
          onClick={() => setActiveView("execute")}
          className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg mb-2 transition-colors ${
            activeView === "execute" ? "bg-blue-600" : "hover:bg-gray-700"
          }`}
        >
          <Code className="h-4 w-4" />
          Execute Code
        </button>
      </nav>
    </div>
  );
};
