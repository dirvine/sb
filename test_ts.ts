#!/usr/bin/env ts-node
/**
 * A sample TypeScript file to test syntax highlighting
 */

import * as fs from 'fs/promises';
import { performance } from 'perf_hooks';

// Interface definitions
interface Config {
    name: string;
    port: number;
    enabled: boolean;
    metadata?: Record<string, unknown>;
}

interface ServerStats {
    uptime: number;
    requests: number;
    errors: number;
}

// Type aliases
type ConfigKey = string;
type ConfigMap = Map<ConfigKey, Config>;
type ValidationResult = { valid: boolean; errors: string[] };

// Enum for server status
enum ServerStatus {
    STOPPED = 'stopped',
    STARTING = 'starting',
    RUNNING = 'running',
    STOPPING = 'stopping',
    ERROR = 'error'
}

// Generic class with constraints
class ConfigManager<T extends Config = Config> {
    private configs: Map<ConfigKey, T> = new Map();
    private _status: ServerStatus = ServerStatus.STOPPED;

    constructor(private readonly maxConfigs: number = 100) {}

    /**
     * Add a configuration with type safety
     */
    public addConfig(key: ConfigKey, config: T): void {
        if (this.configs.size >= this.maxConfigs) {
            throw new Error('Maximum configurations reached');
        }

        const validation = this.validateConfig(config);
        if (!validation.valid) {
            throw new Error(`Invalid config: ${validation.errors.join(', ')}`);
        }

        this.configs.set(key, {
            ...config,
            metadata: {
                ...config.metadata,
                createdAt: new Date().toISOString(),
                version: '1.0'
            }
        });

        console.log(`Added config: ${key} ->`, config);
    }

    /**
     * Get configuration with optional type assertion
     */
    public getConfig<U extends T = T>(key: ConfigKey): U | undefined {
        return this.configs.get(key) as U | undefined;
    }

    /**
     * Validate configuration object
     */
    private validateConfig(config: T): ValidationResult {
        const errors: string[] = [];

        if (!config.name || config.name.trim().length === 0) {
            errors.push('Name cannot be empty');
        }

        if (!config.port || config.port <= 0 || config.port > 65535) {
            errors.push('Port must be between 1 and 65535');
        }

        if (typeof config.enabled !== 'boolean') {
            errors.push('Enabled must be a boolean');
        }

        return { valid: errors.length === 0, errors };
    }

    /**
     * Get all configurations matching a predicate
     */
    public findConfigs(predicate: (config: T) => boolean): T[] {
        return Array.from(this.configs.values()).filter(predicate);
    }

    /**
     * Update configuration using partial update
     */
    public updateConfig(key: ConfigKey, updates: Partial<T>): boolean {
        const existing = this.configs.get(key);
        if (!existing) return false;

        const updated = { ...existing, ...updates };
        const validation = this.validateConfig(updated);
        
        if (!validation.valid) {
            throw new Error(`Update validation failed: ${validation.errors.join(', ')}`);
        }

        this.configs.set(key, updated);
        return true;
    }

    /**
     * Get server statistics
     */
    public getStats(): ServerStats {
        return {
            uptime: performance.now(),
            requests: this.configs.size * 10, // Mock data
            errors: 0
        };
    }

    // Getter/setter for status
    public get status(): ServerStatus {
        return this._status;
    }

    public set status(status: ServerStatus) {
        console.log(`Status changing: ${this._status} -> ${status}`);
        this._status = status;
    }

    /**
     * Export configurations as JSON with proper typing
     */
    public toJSON(): string {
        const data: Record<string, T> = {};
        for (const [key, config] of this.configs) {
            data[key] = config;
        }
        return JSON.stringify(data, null, 2);
    }
}

// Extended interface
interface WebServerConfig extends Config {
    ssl: boolean;
    domain: string;
}

// Async function with proper error handling
async function loadConfigFromFile<T extends Config>(
    filename: string
): Promise<T | null> {
    try {
        const data = await fs.readFile(filename, 'utf8');
        return JSON.parse(data) as T;
    } catch (error: unknown) {
        if (error instanceof Error) {
            console.error(`Failed to load config from ${filename}: ${error.message}`);
        } else {
            console.error(`Unknown error loading ${filename}`);
        }
        return null;
    }
}

// Main function with comprehensive TypeScript features
async function main(): Promise<void> {
    const manager = new ConfigManager<Config>(50);
    manager.status = ServerStatus.STARTING;

    // Sample configurations with proper typing
    const configs: Array<{ key: ConfigKey; config: Config }> = [
        {
            key: 'web',
            config: {
                name: 'webserver',
                port: 8080,
                enabled: true,
                metadata: { type: 'http', version: '2.0' }
            }
        },
        {
            key: 'api',
            config: {
                name: 'apiserver',
                port: 9000,
                enabled: false,
                metadata: { type: 'rest', rateLimit: 1000 }
            }
        },
        {
            key: 'db',
            config: {
                name: 'database',
                port: 5432,
                enabled: true,
                metadata: { type: 'postgresql', poolSize: 10 }
            }
        }
    ];

    // Process configurations with proper error handling
    for (const { key, config } of configs) {
        try {
            manager.addConfig(key, config);
        } catch (error: unknown) {
            const message = error instanceof Error ? error.message : 'Unknown error';
            console.error(`Error adding ${key}: ${message}`);
        }
    }

    manager.status = ServerStatus.RUNNING;

    // Find enabled configurations using predicate
    const enabledConfigs = manager.findConfigs(config => config.enabled);
    console.log(`\nFound ${enabledConfigs.length} enabled configurations`);

    // Display statistics
    const stats = manager.getStats();
    console.log('\nServer Statistics:', {
        ...stats,
        status: manager.status,
        timestamp: new Date().toISOString()
    });

    // Demonstrate partial updates
    const updateResult = manager.updateConfig('api', { enabled: true });
    console.log(`API config update result: ${updateResult}`);

    // Type assertion example
    const webConfig = manager.getConfig<WebServerConfig>('web');
    if (webConfig) {
        console.log(`Web server: ${webConfig.name} on port ${webConfig.port}`);
    }
}

// Execution with proper error handling
if (require.main === module) {
    main()
        .then(() => console.log('Application completed successfully'))
        .catch((error: unknown) => {
            const message = error instanceof Error ? error.message : 'Unknown error';
            console.error('Application error:', message);
            process.exit(1);
        });
}

export { Config, ConfigManager, ServerStatus };