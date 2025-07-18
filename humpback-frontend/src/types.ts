// Types
export interface KVItem {
  key: string;
  kind: "string" | "number" | "boolean" | "json" | "js" | "blob";
  size: number;
  data?: any;
}

export interface ApiResponse {
  status: string;
  data?: any;
  error?: string;
  output?: string;
}