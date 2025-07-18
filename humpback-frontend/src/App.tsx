import React, { useState, useEffect } from "react";
import { Search, Plus, Code, Database } from "lucide-react";
import { ExecuteCodeForm } from "./components/CodeEditor";
import type { ApiResponse, KVItem } from "./types";
import { DatabaseList } from "./components/DatebaseList";
import { Sidebar } from "./components/Sidebar";
import SqlManager from "./components/SqlManager";

// Constants
const API_BASE = "http://localhost:8080";
const TOKEN = "humpback_secret_token_2024";

// API Functions
const api = {
  async request(endpoint: string, data: any): Promise<ApiResponse> {
    const response = await fetch(`${API_BASE}${endpoint}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ token: TOKEN, ...data }),
    });
    return response.json();
  },

  async set(key: string, kind: string, data: string) {
    return this.request("/set", { key, kind, data });
  },

  async get(key: string) {
    return this.request("/get", { key });
  },

  async delete(key: string) {
    return this.request("/delete", { key });
  },

  async list() {
    return this.request("/list", {});
  },

  async exec(key: string) {
    return this.request("/exec", { key });
  },

  async execNow(code: string) {
    return this.request("/execNow", { code });
  },
};

const FloatingScriptButton: React.FC<{
  isOpen: boolean;
  setIsOpen: (open: boolean) => void;
  onExecute: (code: string) => void;
}> = ({ isOpen, setIsOpen, onExecute }) => {
  const [code, setCode] = useState("");

  const handleExecute = () => {
    if (code.trim()) {
      onExecute(code);
      setCode("");
      setIsOpen(false);
    }
  };

  return (
    <div className="fixed bottom-6 right-6 z-50">
      {isOpen && (
        <div className="mb-4 bg-white rounded-lg shadow-lg p-4 w-80 max-w-sm">
          <h3 className="font-semibold mb-2">Quick Script</h3>
          <textarea
            value={code}
            onChange={(e) => setCode(e.target.value)}
            placeholder="Enter JavaScript code..."
            className="w-full h-24 p-2 border rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
            style={{ fontFamily: "monospace" }}
          />
          <div className="flex gap-2 mt-2">
            <button
              onClick={handleExecute}
              className="bg-green-500 hover:bg-green-600 text-white px-3 py-1 rounded text-sm"
            >
              Execute
            </button>
            <button
              onClick={() => setIsOpen(false)}
              className="bg-gray-500 hover:bg-gray-600 text-white px-3 py-1 rounded text-sm"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      <button
        onClick={() => setIsOpen(!isOpen)}
        className="bg-blue-500 hover:bg-blue-600 text-white p-3 rounded-full shadow-lg transition-transform hover:scale-110"
      >
        <Code className="h-6 w-6" />
      </button>
    </div>
  );
};

const AddItemForm: React.FC<{
  onAdd: (key: string, kind: string, data: string) => void;
  isLoading: boolean;
}> = ({ onAdd, isLoading }) => {
  const [key, setKey] = useState("");
  const [kind, setKind] = useState<string>("string");
  const [data, setData] = useState("");

  const handleSubmit = () => {
    if (key.trim() && data.trim()) {
      onAdd(key, kind, data);
      setKey("");
      setData("");
    }
  };

  return (
    <div className="bg-white rounded-lg shadow-lg p-6">
      <h2 className="text-xl font-semibold mb-4">Add New Item</h2>
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium mb-1">Key:</label>
          <input
            type="text"
            value={key}
            onChange={(e) => setKey(e.target.value)}
            className="w-full p-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            maxLength={256}
            required
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Type:</label>
          <select
            value={kind}
            onChange={(e) => setKind(e.target.value)}
            className="w-full p-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="string">String</option>
            <option value="number">Number</option>
            <option value="boolean">Boolean</option>
            <option value="json">JSON</option>
            <option value="js">JavaScript</option>
            <option value="blob">Blob</option>
          </select>
        </div>

        <div>
          <label className="block text-sm font-medium mb-1">Data:</label>
          <textarea
            value={data}
            onChange={(e) => setData(e.target.value)}
            className="w-full p-2 border rounded-lg h-24 resize-vertical focus:outline-none focus:ring-2 focus:ring-blue-500"
            style={{ fontFamily: kind === "js" ? "monospace" : "inherit" }}
            placeholder="Enter your data here..."
            required
          />
        </div>

        <button
          type="button"
          onClick={handleSubmit}
          disabled={isLoading}
          className="w-full bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white py-2 px-4 rounded-lg transition-colors"
        >
          {isLoading ? "Adding..." : "Add Item"}
        </button>
      </div>
    </div>
  );
};

const NotificationToast: React.FC<{
  message: string;
  type: "success" | "error";
  onClose: () => void;
}> = ({ message, type, onClose }) => {
  useEffect(() => {
    const timer = setTimeout(onClose, 5000);
    return () => clearTimeout(timer);
  }, [onClose]);

  return (
    <div
      className={`fixed top-4 right-4 z-50 p-4 rounded-lg shadow-lg ${
        type === "success" ? "bg-green-500" : "bg-red-500"
      } text-white`}
    >
      <div className="flex items-center justify-between">
        <span>{message}</span>
        <button onClick={onClose} className="ml-4 hover:opacity-70">
          ×
        </button>
      </div>
    </div>
  );
};

// Main App Component
const App: React.FC = () => {
  const [activeView, setActiveView] = useState("database");
  const [searchQuery, setSearchQuery] = useState("");
  const [items, setItems] = useState<KVItem[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [notification, setNotification] = useState<{
    message: string;
    type: "success" | "error";
  } | null>(null);
  const [isFloatingOpen, setIsFloatingOpen] = useState(false);

  const showNotification = (message: string, type: "success" | "error") => {
    setNotification({ message, type });
  };
  const [results, setResults] = useState<string[]>([]);

  const loadItems = async () => {
    setIsLoading(true);
    try {
      const result = await api.list();
      if (result.status === "success") {
        setItems(result.data || []);
      } else {
        showNotification(result.error || "Failed to load items", "error");
      }
    } catch (error) {
      showNotification("Network error", "error");
    }
    setIsLoading(false);
  };

  const handleAddItem = async (key: string, kind: string, data: string) => {
    setIsLoading(true);
    try {
      // Validate data based on type
      if (kind === "number" && isNaN(parseFloat(data))) {
        showNotification("Invalid number format", "error");
        setIsLoading(false);
        return;
      }
      if (
        kind === "boolean" &&
        !["true", "false"].includes(data.toLowerCase())
      ) {
        showNotification('Boolean must be "true" or "false"', "error");
        setIsLoading(false);
        return;
      }
      if (kind === "json") {
        try {
          JSON.parse(data);
        } catch {
          showNotification("Invalid JSON format", "error");
          setIsLoading(false);
          return;
        }
      }

      const result = await api.set(key, kind, data);
      if (result.status === "success") {
        showNotification("Item added successfully!", "success");
        loadItems();
      } else {
        showNotification(result.error || "Failed to add item", "error");
      }
    } catch (error) {
      showNotification("Network error", "error");
    }
    setIsLoading(false);
  };

  const handleDeleteItem = async (key: string) => {
    if (!confirm(`Are you sure you want to delete "${key}"?`)) return;

    try {
      const result = await api.delete(key);
      if (result.status === "success") {
        showNotification("Item deleted successfully!", "success");
        loadItems();
      } else {
        showNotification(result.error || "Failed to delete item", "error");
      }
    } catch (error) {
      showNotification("Network error", "error");
    }
  };

  const handleViewItem = async (key: string) => {
    try {
      const result = await api.get(key);
      if (result.status === "success") {
        const data =
          typeof result.data === "object"
            ? JSON.stringify(result.data, null, 2)
            : result.data;
        alert(`Key: ${key}\nData: ${data}`);
      } else {
        showNotification(result.error || "Failed to get item", "error");
      }
    } catch (error) {
      showNotification("Network error", "error");
    }
  };

  const handleExecuteScript = async (key: string) => {
    try {
      const result = await api.exec(key);
      if (result.status === "success") {
        alert(
          `Script executed successfully.\nOutput:\n${
            result.output || "(no output)"
          }`
        );
      } else {
        showNotification(result.error || "Failed to execute script", "error");
      }
    } catch (error) {
      showNotification("Network error", "error");
    }
  };

  const handleExecuteCode = async (code: string) => {
    setIsLoading(true);
    try {
      const result = await api.execNow(code);

      if (result.status === "success") {
        showNotification("Code executed successfully!", "success");
        setResults((prev) => [...prev, JSON.stringify(result.data)]);

        if (result.output) {
          alert(`Output:\n${result.output}`);
        }
      } else {
        showNotification(result.error || "Failed to execute code", "error");
      }
    } catch (error) {
      showNotification("Network error", "error");
    }
    setIsLoading(false);
  };

  useEffect(() => {
    loadItems();
  }, []);

  return (
    <>
      <Sidebar
        activeView={activeView}
        setActiveView={setActiveView}
        searchQuery={searchQuery}
        setSearchQuery={setSearchQuery}
      />

      <main className="mx-[48vh] p-6">
        {activeView === "kv" && (
          <DatabaseList
            items={items}
            searchQuery={searchQuery}
            onRefresh={loadItems}
            onDelete={handleDeleteItem}
            onView={handleViewItem}
            onExecute={handleExecuteScript}
            isLoading={isLoading}
          />
        )}

        {activeView === "add" && (
          <AddItemForm onAdd={handleAddItem} isLoading={isLoading} />
        )}

        {activeView === "execute" && (
          <ExecuteCodeForm
            onExecute={handleExecuteCode}
            isLoading={isLoading}
            resultsResponses={results}
          />
        )}
        {activeView === "sql" && <SqlManager />}
      </main>

      <FloatingScriptButton
        isOpen={isFloatingOpen}
        setIsOpen={setIsFloatingOpen}
        onExecute={handleExecuteCode}
      />

      {notification && (
        <NotificationToast
          message={notification.message}
          type={notification.type}
          onClose={() => setNotification(null)}
        />
      )}
    </>
  );
};

export default App;
