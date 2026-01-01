import { logDebug, logWarn, logError } from './logger';
// Connection Pool for Proxy Requests
interface Connection {
  id: string;
  proxy: string;
  lastUsed: number;
  activeRequests: number;
  maxConcurrent: number;
  isHealthy: boolean;
  responseTime: number;
}

interface PoolConfig {
  maxConnections: number;
  maxConcurrentPerConnection: number;
  healthCheckInterval: number;
  connectionTimeout: number;
  requestTimeout: number;
  idleTimeout: number;
}

export class ProxyConnectionPool {
  private connections: Map<string, Connection> = new Map();
  private requestQueue: Array<{
    proxy: string;
    request: () => Promise<unknown>;
    resolve: (value: unknown) => void;
    reject: (reason?: unknown) => void;
    timestamp: number;
  }> = [];
  
  private config: PoolConfig;
  private healthCheckInterval: ReturnType<typeof setInterval>;
  private cleanupInterval: ReturnType<typeof setInterval>;
  
  constructor(config: Partial<PoolConfig> = {}) {
    this.config = {
      maxConnections: 50,
      maxConcurrentPerConnection: 5,
      healthCheckInterval: 30000, // 30 seconds
      connectionTimeout: 10000, // 10 seconds
      requestTimeout: 30000, // 30 seconds
      idleTimeout: 300000, // 5 minutes
      ...config
    };
    
    this.startHealthChecks();
    this.startCleanup();
  }
  
  // Add a new proxy connection to the pool
  addConnection(proxy: string): void {
    if (this.connections.size >= this.config.maxConnections) {
      logWarn('Connection pool at maximum capacity');
      return;
    }
    
    const connection: Connection = {
      id: this.generateId(),
      proxy,
      lastUsed: Date.now(),
      activeRequests: 0,
      maxConcurrent: this.config.maxConcurrentPerConnection,
      isHealthy: true,
      responseTime: 0
    };
    
    this.connections.set(proxy, connection);
  }
  
  // Execute a request through the connection pool
  async execute<T>(proxy: string, requestFn: () => Promise<T>): Promise<T> {
    // Get or create connection
    let connection = this.connections.get(proxy);
    if (!connection) {
      this.addConnection(proxy);
      connection = this.connections.get(proxy)!;
    }
    
    // Check if connection can handle more requests
    if (connection.activeRequests >= connection.maxConcurrent) {
      // Queue the request
      return new Promise((resolve, reject) => {
        this.requestQueue.push({
          proxy,
          request: requestFn,
          resolve,
          reject,
          timestamp: Date.now()
        });
        this.processQueue();
      });
    }
    
    // Execute request directly
    return this.executeRequest(connection, requestFn);
  }
  
  private async executeRequest<T>(connection: Connection, requestFn: () => Promise<T>): Promise<T> {
    connection.activeRequests++;
    connection.lastUsed = Date.now();
    
    const startTime = Date.now();
    
    try {
      // Add timeout wrapper
      const result = await Promise.race([
        requestFn(),
        this.createTimeoutPromise(this.config.requestTimeout)
      ]);
      
      // Update connection stats
      connection.responseTime = Date.now() - startTime;
      connection.isHealthy = true;
      
      return result;
    } catch (error) {
      connection.isHealthy = false;
      throw error;
    } finally {
      connection.activeRequests--;
      // Process queued requests
      this.processQueue();
    }
  }
  
  private processQueue(): void {
    // Process queued requests in order
    while (this.requestQueue.length > 0) {
      const queued = this.requestQueue[0];
      const connection = this.connections.get(queued.proxy);
      
      if (connection && connection.activeRequests < connection.maxConcurrent) {
        // Remove from queue and execute
        this.requestQueue.shift();
        this.executeRequest(connection, queued.request)
          .then(queued.resolve)
          .catch(queued.reject);
      } else {
        // No available connections for this request
        break;
      }
    }
  }
  
  private createTimeoutPromise(timeout: number): Promise<never> {
    return new Promise((_, reject) => {
      setTimeout(() => reject(new Error('Request timeout')), timeout);
    });
  }
  
  private startHealthChecks(): void {
    this.healthCheckInterval = setInterval(async () => {
      for (const [proxy, connection] of this.connections) {
        if (connection.activeRequests === 0) {
          try {
            // Perform health check
            const startTime = Date.now();
            await fetch('https://httpbin.org/ip', {
              method: 'GET',
              mode: 'no-cors',
              signal: AbortSignal.timeout(5000)
            });
            connection.responseTime = Date.now() - startTime;
            connection.isHealthy = true;
          } catch (error) {
            connection.isHealthy = false;
            logWarn(`Health check failed for proxy ${proxy}:`, error);
          }
        }
      }
    }, this.config.healthCheckInterval);
  }
  
  private startCleanup(): void {
    this.cleanupInterval = setInterval(() => {
      const now = Date.now();
      const toRemove: string[] = [];
      
      for (const [proxy, connection] of this.connections) {
        // Remove unhealthy or idle connections
        if (!connection.isHealthy || 
            (connection.activeRequests === 0 && 
             now - connection.lastUsed > this.config.idleTimeout)) {
          toRemove.push(proxy);
        }
      }
      
      toRemove.forEach(proxy => {
        this.connections.delete(proxy);
      });
      
      // Clean up old queued requests
      this.requestQueue = this.requestQueue.filter(
        req => now - req.timestamp < this.config.requestTimeout
      );
    }, 60000); // Every minute
  }
  
  // Get pool statistics
  getStats() {
    const activeConnections = Array.from(this.connections.values());
    const healthyConnections = activeConnections.filter(c => c.isHealthy);
    const totalActiveRequests = activeConnections.reduce((sum, c) => sum + c.activeRequests, 0);
    const avgResponseTime = healthyConnections.length > 0
      ? healthyConnections.reduce((sum, c) => sum + c.responseTime, 0) / healthyConnections.length
      : 0;
    
    return {
      totalConnections: this.connections.size,
      healthyConnections: healthyConnections.length,
      activeRequests: totalActiveRequests,
      queuedRequests: this.requestQueue.length,
      averageResponseTime: avgResponseTime,
      utilization: totalActiveRequests / (this.connections.size * this.config.maxConcurrentPerConnection)
    };
  }
  
  // Optimize connection pool based on usage patterns
  optimize(): void {
    const stats = this.getStats();
    
    // Add more connections if utilization is high
    if (stats.utilization > 0.8 && this.connections.size < this.config.maxConnections) {
      logDebug('High utilization detected, consider adding more connections');
    }
    
    // Remove underutilized connections
    if (stats.utilization < 0.2 && this.connections.size > 10) {
      const toRemove = Math.floor(this.connections.size * 0.3);
      let removed = 0;
      
      for (const [proxy, connection] of this.connections) {
        if (removed >= toRemove) break;
        if (connection.activeRequests === 0 && connection.lastUsed < Date.now() - 60000) {
          this.connections.delete(proxy);
          removed++;
        }
      }
    }
  }
  
  private generateId(): string {
    return Math.random().toString(36).substr(2, 9);
  }
  
  // Destroy the pool and clean up resources
  destroy(): void {
    clearInterval(this.healthCheckInterval);
    clearInterval(this.cleanupInterval);
    this.connections.clear();
    this.requestQueue.length = 0;
  }
}

// Singleton instance for the application
export const proxyPool = new ProxyConnectionPool({
  maxConnections: 30,
  maxConcurrentPerConnection: 3,
  healthCheckInterval: 20000, // 20 seconds
  connectionTimeout: 8000,
  requestTimeout: 25000,
  idleTimeout: 180000 // 3 minutes
});

// Enhanced API wrapper with connection pooling
export async function pooledRequest<T>(proxy: string, requestFn: () => Promise<T>): Promise<T> {
  return proxyPool.execute(proxy, requestFn);
}

// Batch multiple requests across different proxies
export async function batchPooledRequests<T>(
  requests: Array<{ proxy: string; request: () => Promise<T> }>
): Promise<T[]> {
  const promises = requests.map(({ proxy, request }) => 
    pooledRequest(proxy, request)
  );
  return Promise.all(promises);
}
