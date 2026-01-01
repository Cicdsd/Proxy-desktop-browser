/**
 * Structured logging utility for the frontend application
 * Provides consistent logging with levels, timestamps, and optional backend integration
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: Date;
  context?: Record<string, unknown>;
  source?: string;
}

class Logger {
  private static instance: Logger;
  private minLevel: LogLevel = LogLevel.INFO;
  private logHistory: LogEntry[] = [];
  private maxHistorySize = 500;

  private constructor() {
    // Set debug level in development
    if (this.isDevelopment()) {
      this.minLevel = LogLevel.DEBUG;
    }
  }

  static getInstance(): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger();
    }
    return Logger.instance;
  }

  private isDevelopment(): boolean {
    return typeof window !== 'undefined' && 
           (window.location.hostname === 'localhost' ||
            window.location.hostname === '127.0.0.1');
  }

  private formatMessage(level: LogLevel, message: string, context?: Record<string, unknown>): string {
    const levelStr = LogLevel[level];
    const timestamp = new Date().toISOString();
    const contextStr = context ? ` ${JSON.stringify(context)}` : '';
    return `[${timestamp}] [${levelStr}] ${message}${contextStr}`;
  }

  private log(level: LogLevel, message: string, context?: Record<string, unknown>, source?: string): void {
    if (level < this.minLevel) return;

    const entry: LogEntry = {
      level,
      message,
      timestamp: new Date(),
      context,
      source,
    };

    // Store in history
    this.logHistory.push(entry);
    if (this.logHistory.length > this.maxHistorySize) {
      this.logHistory.shift();
    }

    const formattedMessage = this.formatMessage(level, message, context);

    // Output to appropriate console method
    switch (level) {
      case LogLevel.DEBUG:
        console.debug(formattedMessage);
        break;
      case LogLevel.INFO:
        console.info(formattedMessage);
        break;
      case LogLevel.WARN:
        console.warn(formattedMessage);
        break;
      case LogLevel.ERROR:
        console.error(formattedMessage);
        break;
    }
  }

  debug(message: string, context?: Record<string, unknown>, source?: string): void {
    this.log(LogLevel.DEBUG, message, context, source);
  }

  info(message: string, context?: Record<string, unknown>, source?: string): void {
    this.log(LogLevel.INFO, message, context, source);
  }

  warn(message: string, context?: Record<string, unknown>, source?: string): void {
    this.log(LogLevel.WARN, message, context, source);
  }

  error(message: string, context?: Record<string, unknown>, source?: string): void {
    this.log(LogLevel.ERROR, message, context, source);
  }

  getHistory(): LogEntry[] {
    return [...this.logHistory];
  }

  setMinLevel(level: LogLevel): void {
    this.minLevel = level;
  }

  clearHistory(): void {
    this.logHistory = [];
  }
}

// Export singleton instance
export const logger = Logger.getInstance();

// Export convenience functions
export const logDebug = (message: string, context?: Record<string, unknown>) => logger.debug(message, context);
export const logInfo = (message: string, context?: Record<string, unknown>) => logger.info(message, context);
export const logWarn = (message: string, context?: Record<string, unknown>) => logger.warn(message, context);
export const logError = (message: string, context?: Record<string, unknown>) => logger.error(message, context);
