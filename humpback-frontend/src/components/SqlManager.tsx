import React, { useState, useEffect } from "react";
import {
  Plus,
  Trash2,
  Play,
  Database,
  Table,
  Search,
  AlertCircle,
  CheckCircle,
} from "lucide-react";
import SqlApi, { type SqlExecResponse, type SqlQueryResponse } from "./sqlApi";

interface Column {
  name: string;
  type: "TEXT" | "INTEGER" | "REAL" | "BLOB" | "NUMERIC";
  primaryKey: boolean;
  notNull: boolean;
  defaultValue: string;
}

interface TableInfo {
  name: string;
}

const SqlManager: React.FC = () => {
  const [activeTab, setActiveTab] = useState<"create" | "query" | "exec">(
    "create"
  );
  const [tables, setTables] = useState<TableInfo[]>([]);
  const [queryResult, setQueryResult] = useState<SqlQueryResponse | null>(null);
  const [execResult, setExecResult] = useState<SqlExecResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  // Create Table Form State
  const [tableName, setTableName] = useState("");
  const [columns, setColumns] = useState<Column[]>([
    {
      name: "",
      type: "TEXT",
      primaryKey: false,
      notNull: false,
      defaultValue: "",
    },
  ]);

  // Query/Exec State
  const [queryText, setQueryText] = useState("");
  const [execText, setExecText] = useState("");

  const columnTypes: Array<"TEXT" | "INTEGER" | "REAL" | "BLOB" | "NUMERIC"> = [
    "TEXT",
    "INTEGER",
    "REAL",
    "BLOB",
    "NUMERIC",
  ];

  // Initialize the real API client
  const sqlApi = new SqlApi();

  const showMessage = (message: string, type: "info" | "error" = "info") => {
    if (type === "error") {
      setError(message);
      setSuccess("");
    } else {
      setSuccess(message);
      setError("");
    }
    setTimeout(() => {
      setError("");
      setSuccess("");
    }, 5000);
  };

  useEffect(() => {
    loadTables();
  }, []);

  const loadTables = async () => {
    try {
      setLoading(true);
      const tableList = await sqlApi.listTables();
      setTables(tableList);
    } catch (err) {
      console.error("Failed to load tables:", err);
      showMessage("Failed to load tables", "error");
    } finally {
      setLoading(false);
    }
  };

  const addColumn = () => {
    setColumns([
      ...columns,
      {
        name: "",
        type: "TEXT",
        primaryKey: false,
        notNull: false,
        defaultValue: "",
      },
    ]);
  };

  const removeColumn = (index: number) => {
    setColumns(columns.filter((_, i) => i !== index));
  };

  const updateColumn = (index: number, field: keyof Column, value: any) => {
    const newColumns = [...columns];
    newColumns[index] = { ...newColumns[index], [field]: value };
    setColumns(newColumns);
  };

  const generateCreateTableSQL = (): string => {
    if (!tableName.trim()) return "";

    const columnDefs = columns
      .filter((col) => col.name.trim())
      .map((col) => {
        let def = `${col.name} ${col.type}`;
        if (col.primaryKey) def += " PRIMARY KEY";
        if (col.notNull) def += " NOT NULL";
        if (col.defaultValue.trim()) def += ` DEFAULT '${col.defaultValue}'`;
        return def;
      });

    return `CREATE TABLE ${tableName} (\n  ${columnDefs.join(",\n  ")}\n);`;
  };

  const handleCreateTable = async () => {
    const sql = generateCreateTableSQL();
    if (!sql) {
      showMessage("Please enter a table name and at least one column", "error");
      return;
    }

    try {
      setLoading(true);
      const result = await sqlApi.exec(sql);
      showMessage("Table created successfully!");
      setTableName("");
      setColumns([
        {
          name: "",
          type: "TEXT",
          primaryKey: false,
          notNull: false,
          defaultValue: "",
        },
      ]);
      await loadTables();
    } catch (err) {
      console.error("Failed to create table:", err);
      showMessage(
        `Failed to create table: ${
          err instanceof Error ? err.message : "Unknown error"
        }`,
        "error"
      );
    } finally {
      setLoading(false);
    }
  };

  const handleQuery = async () => {
    if (!queryText.trim()) {
      showMessage("Please enter a query", "error");
      return;
    }

    try {
      setLoading(true);
      const result = await sqlApi.query(queryText);
      setQueryResult(result);
      showMessage(
        `Query executed successfully. ${result.rows_count} rows returned.`
      );
    } catch (err) {
      console.error("Query execution failed:", err);
      showMessage(
        `Query execution failed: ${
          err instanceof Error ? err.message : "Unknown error"
        }`,
        "error"
      );
    } finally {
      setLoading(false);
    }
  };

  const handleExec = async () => {
    if (!execText.trim()) {
      showMessage("Please enter a statement", "error");
      return;
    }

    try {
      setLoading(true);
      const result = await sqlApi.exec(execText);
      setExecResult(result);
      showMessage(result.message);
      await loadTables(); // Refresh tables in case structure changed
    } catch (err) {
      console.error("Statement execution failed:", err);
      showMessage(
        `Statement execution failed: ${
          err instanceof Error ? err.message : "Unknown error"
        }`,
        "error"
      );
    } finally {
      setLoading(false);
    }
  };

  const renderTable = (data: any[] | null) => {
    if (!data || !Array.isArray(data) || data.length === 0) {
      return (
        <div className="text-center py-4 text-gray-500">No data to display</div>
      );
    }

    const headers = Object.keys(data[0]);

    return (
      <div className="overflow-x-auto">
        <table className="w-full border-collapse border border-gray-300">
          <thead>
            <tr className="bg-gray-50">
              {headers.map((header) => (
                <th
                  key={header}
                  className="border border-gray-300 px-4 py-2 text-left font-medium"
                >
                  {header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {data.map((row, index) => (
              <tr key={index} className="hover:bg-gray-50">
                {headers.map((header) => (
                  <td key={header} className="border border-gray-300 px-4 py-2">
                    {row[header]}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  };

  return (
    <div className="max-w-6xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow-lg">
        <div className="border-b">
          <div className="flex space-x-1">
            {["create", "query", "exec"].map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                className={`px-6 py-3 font-medium text-sm transition-colors ${
                  activeTab === tab
                    ? "border-b-2 border-blue-500 text-blue-600 bg-blue-50"
                    : "text-gray-500 hover:text-gray-700"
                }`}
              >
                {tab === "create" && (
                  <Database className="inline w-4 h-4 mr-2" />
                )}
                {tab === "query" && <Search className="inline w-4 h-4 mr-2" />}
                {tab === "exec" && <Play className="inline w-4 h-4 mr-2" />}
                {tab.charAt(0).toUpperCase() + tab.slice(1)}
                {tab === "create" && " Table"}
                {tab === "query" && " Data"}
                {tab === "exec" && " Statement"}
              </button>
            ))}
          </div>
        </div>

        <div className="p-6">
          {/* Messages */}
          {error && (
            <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg flex items-center">
              <AlertCircle className="w-5 h-5 text-red-500 mr-2" />
              <span className="text-red-700">{error}</span>
            </div>
          )}
          {success && (
            <div className="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg flex items-center">
              <CheckCircle className="w-5 h-5 text-green-500 mr-2" />
              <span className="text-green-700">{success}</span>
            </div>
          )}

          {/* Create Table Tab */}
          {activeTab === "create" && (
            <div className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Table Name
                </label>
                <input
                  type="text"
                  value={tableName}
                  onChange={(e) => setTableName(e.target.value)}
                  className="w-full p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Enter table name"
                />
              </div>

              <div>
                <div className="flex items-center justify-between mb-4">
                  <label className="block text-sm font-medium text-gray-700">
                    Columns
                  </label>
                  <button
                    onClick={addColumn}
                    className="flex items-center px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
                  >
                    <Plus className="w-4 h-4 mr-2" />
                    Add Column
                  </button>
                </div>

                <div className="space-y-4">
                  {columns.map((column, index) => (
                    <div
                      key={index}
                      className="p-4 border border-gray-200 rounded-lg"
                    >
                      <div className="grid grid-cols-1 md:grid-cols-6 gap-4">
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Column Name
                          </label>
                          <input
                            type="text"
                            value={column.name}
                            onChange={(e) =>
                              updateColumn(index, "name", e.target.value)
                            }
                            className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            placeholder="column_name"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Type
                          </label>
                          <select
                            value={column.type}
                            onChange={(e) =>
                              updateColumn(index, "type", e.target.value)
                            }
                            className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                          >
                            {columnTypes.map((type) => (
                              <option key={type} value={type}>
                                {type}
                              </option>
                            ))}
                          </select>
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-600 mb-1">
                            Default Value
                          </label>
                          <input
                            type="text"
                            value={column.defaultValue}
                            onChange={(e) =>
                              updateColumn(
                                index,
                                "defaultValue",
                                e.target.value
                              )
                            }
                            className="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            placeholder="default"
                          />
                        </div>
                        <div className="flex items-center space-x-4">
                          <label className="flex items-center">
                            <input
                              type="checkbox"
                              checked={column.primaryKey}
                              onChange={(e) =>
                                updateColumn(
                                  index,
                                  "primaryKey",
                                  e.target.checked
                                )
                              }
                              className="mr-2"
                            />
                            <span className="text-xs">Primary Key</span>
                          </label>
                        </div>
                        <div className="flex items-center space-x-4">
                          <label className="flex items-center">
                            <input
                              type="checkbox"
                              checked={column.notNull}
                              onChange={(e) =>
                                updateColumn(index, "notNull", e.target.checked)
                              }
                              className="mr-2"
                            />
                            <span className="text-xs">Not Null</span>
                          </label>
                        </div>
                        <div className="flex items-center justify-end">
                          <button
                            onClick={() => removeColumn(index)}
                            className="p-2 text-red-500 hover:bg-red-50 rounded"
                            disabled={columns.length === 1}
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Generated SQL
                </label>
                <pre className="bg-gray-50 p-4 rounded-lg border text-sm overflow-x-auto">
                  {generateCreateTableSQL() ||
                    "Enter table name and columns to generate SQL"}
                </pre>
              </div>

              <button
                onClick={handleCreateTable}
                disabled={loading}
                className="w-full bg-green-500 hover:bg-green-600 disabled:bg-gray-400 text-white py-3 px-6 rounded-lg transition-colors flex items-center justify-center"
              >
                <Table className="w-5 h-5 mr-2" />
                {loading ? "Creating..." : "Create Table"}
              </button>
            </div>
          )}

          {/* Query Tab */}
          {activeTab === "query" && (
            <div className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  SQL Query
                </label>
                <textarea
                  value={queryText}
                  onChange={(e) => setQueryText(e.target.value)}
                  className="w-full h-32 p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="SELECT * FROM table_name;"
                />
              </div>

              <button
                onClick={handleQuery}
                disabled={loading}
                className="bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white py-3 px-6 rounded-lg transition-colors flex items-center"
              >
                <Play className="w-5 h-5 mr-2" />
                {loading ? "Executing..." : "Execute Query"}
              </button>

              {queryResult && (
                <div>
                  <h3 className="text-lg font-medium mb-4">Query Results</h3>
                  <div className="bg-gray-50 p-4 rounded-lg">
                    {renderTable(queryResult.data)}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Exec Tab */}
          {activeTab === "exec" && (
            <div className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  SQL Statement
                </label>
                <textarea
                  value={execText}
                  onChange={(e) => setExecText(e.target.value)}
                  className="w-full h-32 p-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="INSERT INTO table_name (column1, column2) VALUES ('value1', 'value2');"
                />
              </div>

              <button
                onClick={handleExec}
                disabled={loading}
                className="bg-orange-500 hover:bg-orange-600 disabled:bg-gray-400 text-white py-3 px-6 rounded-lg transition-colors flex items-center"
              >
                <Play className="w-5 h-5 mr-2" />
                {loading ? "Executing..." : "Execute Statement"}
              </button>

              {execResult && (
                <div>
                  <h3 className="text-lg font-medium mb-4">Execution Result</h3>
                  <div className="bg-gray-50 p-4 rounded-lg">
                    <p className="text-sm">
                      <strong>Message:</strong> {execResult.message}
                    </p>
                    {execResult.rows_affected !== null && (
                      <p className="text-sm mt-2">
                        <strong>Rows affected:</strong>{" "}
                        {execResult.rows_affected}
                      </p>
                    )}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Tables List */}
          {tables.length > 0 && (
            <div className="mt-8 pt-6 border-t">
              <h3 className="text-lg font-medium mb-4">Existing Tables</h3>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {tables.map((table) => (
                  <div
                    key={table.name}
                    className="p-4 border border-gray-200 rounded-lg"
                  >
                    <div className="flex items-center">
                      <Table className="w-5 h-5 text-gray-500 mr-2" />
                      <span className="font-medium">{table.name}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default SqlManager;
