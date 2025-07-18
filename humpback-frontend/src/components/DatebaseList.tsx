import { Eye, Play, RefreshCw, Trash2 } from "lucide-react";
import type { KVItem } from "../types";

export const DatabaseList: React.FC<{
  items: KVItem[];
  searchQuery: string;
  onRefresh: () => void;
  onDelete: (key: string) => void;
  onView: (key: string) => void;
  onExecute: (key: string) => void;
  isLoading: boolean;
}> = ({
  items,
  searchQuery,
  onRefresh,
  onDelete,
  onView,
  onExecute,
  isLoading,
}) => {
  const filteredItems = items.filter((item) =>
    item.key.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="bg-white rounded-lg shadow-lg p-6">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-semibold">Database Items</h2>
        <button
          onClick={onRefresh}
          disabled={isLoading}
          className="bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white py-2 px-4 rounded-lg transition-colors flex items-center gap-2"
        >
          <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
          {isLoading ? "Loading..." : "Refresh"}
        </button>
      </div>

      <div className="max-h-96 overflow-y-auto border rounded-lg">
        {filteredItems.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            {searchQuery
              ? "No items match your search"
              : "No items in database"}
          </div>
        ) : (
          filteredItems.map((item) => (
            <div
              key={item.key}
              className="flex items-center justify-between p-4 border-b last:border-b-0 hover:bg-gray-50"
            >
              <div className="flex-1">
                <div className="font-medium">{item.key}</div>
                <div className="text-sm text-gray-500">
                  Type: {item.kind.toUpperCase()} | Size: {item.size} bytes
                </div>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => onView(item.key)}
                  className="p-2 text-blue-500 hover:bg-blue-50 rounded"
                  title="View"
                >
                  <Eye className="h-4 w-4" />
                </button>
                <button
                  onClick={() => onDelete(item.key)}
                  className="p-2 text-red-500 hover:bg-red-50 rounded"
                  title="Delete"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
                {item.kind === "js" && (
                  <button
                    onClick={() => onExecute(item.key)}
                    className="p-2 text-green-500 hover:bg-green-50 rounded"
                    title="Execute"
                  >
                    <Play className="h-4 w-4" />
                  </button>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
