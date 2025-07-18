// SQL API Integration Functions - TypeScript
// Replace the mock API in the React component with these functions

interface SqlQueryResponse {
  data: any[];
  rows_count: number;
}

interface SqlExecResponse {
  message: string;
  rows_affected: number | null;
}

interface ApiSuccessResponse<T> {
  status: 'success';
  data: T;
}

interface ApiErrorResponse {
  status: 'error';
  error: string;
}

type ApiResponse<T> = ApiSuccessResponse<T> | ApiErrorResponse;

const SQL_API_BASE_URL = 'http://localhost:8080'; // Adjust to your backend URL
const AUTH_TOKEN = 'humpback_secret_token_2024';

class SqlApi {
  private baseUrl: string;
  private token: string;

  constructor(baseUrl: string = SQL_API_BASE_URL, token: string = AUTH_TOKEN) {
    this.baseUrl = baseUrl;
    this.token = token;
  }

  async query(queryText: string): Promise<SqlQueryResponse> {
    const response = await fetch(`${this.baseUrl}/sql/query`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        token: this.token,
        query: queryText,
      }),
    });

    if (!response.ok) {
      const error: ApiErrorResponse = await response.json();
      throw new Error(error.error || 'Query failed');
    }

    const result: ApiResponse<SqlQueryResponse> = await response.json();
    if (result.status === 'success') {
      return result.data;
    } else {
      throw new Error(result.error || 'Query failed');
    }
  }

  async exec(statement: string): Promise<SqlExecResponse> {
    const response = await fetch(`${this.baseUrl}/sql/exec`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        token: this.token,
        statement: statement,
      }),
    });

    if (!response.ok) {
      const error: ApiErrorResponse = await response.json();
      throw new Error(error.error || 'Statement execution failed');
    }

    const result: ApiResponse<SqlExecResponse> = await response.json();
    if (result.status === 'success') {
      return result.data;
    } else {
      throw new Error(result.error || 'Statement execution failed');
    }
  }

  async listTables(): Promise<Array<{ name: string }>> {
    const response = await fetch(`${this.baseUrl}/sql/tables`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const error: ApiErrorResponse = await response.json();
      throw new Error(error.error || 'Failed to list tables');
    }

    const result: ApiResponse<Array<{ name: string }>> = await response.json();
    if (result.status === 'success') {
      return result.data || [];
    } else {
      throw new Error(result.error || 'Failed to list tables');
    }
  }

  async getTableInfo(tableName: string): Promise<any[]> {
    const response = await fetch(`${this.baseUrl}/sql/table/${tableName}/info`, {
      method: 'GET',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const error: ApiErrorResponse = await response.json();
      throw new Error(error.error || 'Failed to get table info');
    }

    const result: ApiResponse<any[]> = await response.json();
    if (result.status === 'success') {
      return result.data || [];
    } else {
      throw new Error(result.error || 'Failed to get table info');
    }
  }

  // Helper methods for common operations
  async insertData(tableName: string, data: Record<string, any>): Promise<SqlExecResponse> {
    const columns = Object.keys(data).join(', ');
    const values = Object.values(data)
      .map(v => typeof v === 'string' ? `'${v.replace(/'/g, "''")}'` : v)
      .join(', ');
    const statement = `INSERT INTO ${tableName} (${columns}) VALUES (${values})`;
    return await this.exec(statement);
  }

  async updateData(tableName: string, data: Record<string, any>, whereClause: string): Promise<SqlExecResponse> {
    const setClause = Object.entries(data)
      .map(([key, value]) => `${key} = ${typeof value === 'string' ? `'${value.replace(/'/g, "''")}'` : value}`)
      .join(', ');
    const statement = `UPDATE ${tableName} SET ${setClause} WHERE ${whereClause}`;
    return await this.exec(statement);
  }

  async deleteData(tableName: string, whereClause: string): Promise<SqlExecResponse> {
    const statement = `DELETE FROM ${tableName} WHERE ${whereClause}`;
    return await this.exec(statement);
  }

  async dropTable(tableName: string): Promise<SqlExecResponse> {
    const statement = `DROP TABLE IF EXISTS ${tableName}`;
    return await this.exec(statement);
  }
}

export default SqlApi;
export type { SqlQueryResponse, SqlExecResponse, ApiSuccessResponse, ApiErrorResponse };

 