// Advanced error handling system with retry mechanisms

import { invoke } from '@tauri-apps/api/tauri';
import { logInfo, logWarn, logError, logDebug } from './logger';
import { performanceStore } from './stores';

// Development mode detection for Tauri
const isDevelopment = () => {
  // Check for Tauri development environment
  return typeof window !== 'undefined' && 
         (window.__TAURI_METADATA__?.__currentWindow?.label === 'main' ||
          window.location.hostname === 'localhost' ||
          window.location.hostname === '127.0.0.1');
};

export interface RetryOptions {
  maxAttempts?: number;
  baseDelay?: number;
  maxDelay?: number;
  backoffFactor?: number;
  jitter?: boolean;
  retryCondition?: (error: unknown) => boolean;
}

export interface ErrorContext {
  operation: string;
  timestamp: number;
  attempt: number;
  error: Error;
  context?: Record<string, any>;
}

class AdvancedErrorHandler {
  private errorLog: ErrorContext[] = [];
  private maxLogSize = 1000;
  private retryStrategies = new Map<string, RetryOptions>();
  private lastLogTimes = new Map<string, number>();
  private logThrottleMs = 5000; // Throttle identical errors to once per 5 seconds

  // Default retry configuration
  private defaultRetryOptions: Required<RetryOptions> = {
    maxAttempts: 3,
    baseDelay: 1000,
    maxDelay: 10000,
    backoffFactor: 2,
    jitter: true,
    retryCondition: (error) => this.shouldRetry(error)
  };

  /**
   * Execute operation with advanced retry logic
   */
  async executeWithRetry<T>(
    operation: () => Promise<T>,
    operationName: string,
    options: RetryOptions = {}
  ): Promise<T> {
    const opts = { ...this.defaultRetryOptions, ...options };
    const strategyKey = `${operationName}_${JSON.stringify(options)}`;
    
    // Store or retrieve retry strategy
    if (!this.retryStrategies.has(strategyKey)) {
      this.retryStrategies.set(strategyKey, opts);
    }

    let lastError: Error;
    const startTime = performance.now();

    for (let attempt = 1; attempt <= opts.maxAttempts; attempt++) {
      try {
        const result = await operation();
        
        // Log success if it took multiple attempts
        if (attempt > 1) {
          this.logSuccess(operationName, attempt, performance.now() - startTime);
        }
        
        return result;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        
        // Log error context
        this.logError({
          operation: operationName,
          timestamp: Date.now(),
          attempt,
          error: lastError,
          context: { willRetry: attempt < opts.maxAttempts }
        });

        // Check if we should retry
        if (attempt === opts.maxAttempts || !opts.retryCondition(lastError)) {
          break;
        }

        // Calculate delay with exponential backoff and jitter
        const delay = this.calculateDelay(attempt, opts);
        await this.sleep(delay);
      }
    }

    // All attempts failed - throw enhanced error
    throw new Error(
      `Operation '${operationName}' failed after ${opts.maxAttempts} attempts. Last error: ${lastError.message}`
    );
  }

  /**
   * Enhanced Tauri invoke with retry and monitoring
   */
  async invokeWithRetry<T>(
    command: string,
    args?: Record<string, unknown>,
    options: RetryOptions = {}
  ): Promise<T> {
    // Update performance metrics
    performanceStore.update(p => ({ ...p, apiCalls: p.apiCalls + 1 }));

    return this.executeWithRetry(
      () => invoke<T>(command, args),
      `tauri_invoke_${command}`,
      options
    );
  }

  /**
   * Circuit breaker pattern for preventing cascade failures
   */
  createCircuitBreaker<T>(
    operation: () => Promise<T>,
    options: {
      failureThreshold?: number;
      resetTimeout?: number;
      monitoringPeriod?: number;
    } = {}
  ) {
    const {
      failureThreshold = 5,
      resetTimeout = 60000,
      monitoringPeriod = 10000
    } = options;

    let failureCount = 0;
    let lastFailureTime = 0;
    let state: 'CLOSED' | 'OPEN' | 'HALF_OPEN' = 'CLOSED';

    return async (): Promise<T> => {
      const now = Date.now();

      // Check if we should attempt to reset
      if (state === 'OPEN' && now - lastFailureTime > resetTimeout) {
        state = 'HALF_OPEN';
        logInfo(`Circuit breaker entering HALF_OPEN state`);
      }

      // Reject if circuit is open
      if (state === 'OPEN') {
        throw new Error('Circuit breaker is OPEN - operation rejected');
      }

      try {
        const result = await operation();
        
        // Reset on success in HALF_OPEN state
        if (state === 'HALF_OPEN') {
          state = 'CLOSED';
          failureCount = 0;
          logInfo(`Circuit breaker reset to CLOSED state`);
        }
        
        return result;
      } catch (error) {
        failureCount++;
        lastFailureTime = now;

        // Open circuit if threshold exceeded
        if (failureCount >= failureThreshold) {
          state = 'OPEN';
          logError(`Circuit breaker opened after ${failureCount} failures`);
        }

        throw error;
      }
    };
  }

  /**
   * Bulkhead pattern for resource isolation
   */
  createBulkhead<T>(
    operation: () => Promise<T>,
    maxConcurrent: number = 10
  ) {
    let running = 0;
    const queue: Array<{
      resolve: (value: T) => void;
      reject: (error: Error) => void;
    }> = [];

    return async (): Promise<T> => {
      return new Promise<T>((resolve, reject) => {
        queue.push({ resolve, reject });
        processQueue();
      });

      async function processQueue() {
        if (running >= maxConcurrent || queue.length === 0) return;

        running++;
        const { resolve, reject } = queue.shift()!;

        try {
          const result = await operation();
          resolve(result);
        } catch (error) {
          reject(error instanceof Error ? error : new Error(String(error)));
        } finally {
          running--;
          processQueue();
        }
      }
    };
  }

  /**
   * Timeout wrapper for operations
   */
  withTimeout<T>(
    operation: () => Promise<T>,
    timeoutMs: number,
    timeoutError: string = 'Operation timed out'
  ): Promise<T> {
    return Promise.race([
      operation(),
      new Promise<never>((_, reject) => {
        setTimeout(() => reject(new Error(timeoutError)), timeoutMs);
      })
    ]);
  }

  /**
   * Get error statistics
   */
  getErrorStats(): {
    totalErrors: number;
    errorsByOperation: Record<string, number>;
    recentErrors: ErrorContext[];
    errorRate: number;
  } {
    const errorsByOperation: Record<string, number> = {};
    const recentTime = Date.now() - 300000; // Last 5 minutes
    let recentErrorCount = 0;

    for (const error of this.errorLog) {
      errorsByOperation[error.operation] = (errorsByOperation[error.operation] || 0) + 1;
      if (error.timestamp > recentTime) {
        recentErrorCount++;
      }
    }

    return {
      totalErrors: this.errorLog.length,
      errorsByOperation,
      recentErrors: this.errorLog.slice(-10),
      errorRate: recentErrorCount / (300000 / 1000) // Errors per second in last 5 minutes
    };
  }

  /**
   * Clear error log
   */
  clearErrors(): void {
    this.errorLog = [];
  }

  private calculateDelay(attempt: number, options: Required<RetryOptions>): number {
    let delay = Math.min(
      options.baseDelay * Math.pow(options.backoffFactor, attempt - 1),
      options.maxDelay
    );

    if (options.jitter) {
      delay *= (0.5 + Math.random() * 0.5);
    }

    return delay;
  }

  private shouldRetry(error: unknown): boolean {
    // Retry on network errors, timeouts, and 5xx server errors
    if (error.name === 'NetworkError' || error.name === 'TimeoutError') {
      return true;
    }

    // Check for specific error codes that should be retried
    if (error.code) {
      const retryableCodes = ['ECONNRESET', 'ETIMEDOUT', 'ENOTFOUND'];
      return retryableCodes.includes(error.code);
    }

    // Check HTTP status if available
    if (error.status) {
      return error.status >= 500 || error.status === 429; // Retry on server errors and rate limits
    }

    // Default: don't retry on client errors (4xx) or unknown errors
    return false;
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  private logError(context: ErrorContext): void {
    this.errorLog.push(context);
    
    // Maintain log size
    if (this.errorLog.length > this.maxLogSize) {
      this.errorLog = this.errorLog.slice(-this.maxLogSize);
    }

    // Throttled logging to prevent console flooding
    const errorKey = `${context.operation}_${context.error.message}`;
    const now = Date.now();
    const lastLogTime = this.lastLogTimes.get(errorKey);
    
    if (isDevelopment() && (!lastLogTime || now - lastLogTime > this.logThrottleMs)) {
      logError(`[Error] ${context.operation} (attempt ${context.attempt}):`, context.error);
      this.lastLogTimes.set(errorKey, now);
      
      // Clean old entries from throttle map
      if (this.lastLogTimes.size > 100) {
        const cutoff = now - this.logThrottleMs * 2;
        for (const [key, time] of this.lastLogTimes.entries()) {
          if (time < cutoff) {
            this.lastLogTimes.delete(key);
          }
        }
      }
    }
  }

  private logSuccess(operation: string, attempts: number, duration: number): void {
    if (isDevelopment()) {
      logInfo(
        `[Success] ${operation} succeeded after ${attempts} attempts in ${duration.toFixed(2)}ms`
      );
    }
  }
}

// Singleton instance
export const errorHandler = new AdvancedErrorHandler();

// Convenience functions
export const invokeWithRetry = <T>(command: string, args?: Record<string, unknown>, options?: RetryOptions) =>
  errorHandler.invokeWithRetry<T>(command, args, options);

export const executeWithRetry = <T>(
  operation: () => Promise<T>,
  operationName: string,
  options?: RetryOptions
) => errorHandler.executeWithRetry(operation, operationName, options);

// Enhanced error types
export class NetworkError extends Error {
  constructor(message: string, public code?: string) {
    super(message);
    this.name = 'NetworkError';
  }
}

export class TimeoutError extends Error {
  constructor(message: string = 'Operation timed out') {
    super(message);
    this.name = 'TimeoutError';
  }
}

export class ValidationError extends Error {
  constructor(message: string, public field?: string) {
    super(message);
    this.name = 'ValidationError';
  }
}
