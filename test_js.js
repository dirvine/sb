#!/usr/bin/env node
/**
 * A sample JavaScript file to test syntax highlighting
 */

const fs = require('fs');
const path = require('path');
const { performance } = require('perf_hooks');

// ES6 class definition
class ConfigManager {
    constructor() {
        this.configs = new Map();
        this._initialized = false;
    }

    /**
     * Add a configuration
     * @param {string} key - Configuration key
     * @param {Object} config - Configuration object
     */
    addConfig(key, config) {
        if (!this.isValidConfig(config)) {
            throw new Error(`Invalid config for '${key}'`);
        }
        
        this.configs.set(key, {
            ...config,
            timestamp: Date.now()
        });
        
        console.log(`Added config: ${key} ->`, config);
    }

    /**
     * Get configuration by key
     * @param {string} key - Configuration key
     * @returns {Object|null} Configuration object or null
     */
    getConfig(key) {
        return this.configs.get(key) || null;
    }

    /**
     * Validate configuration object
     * @param {Object} config - Configuration to validate
     * @returns {boolean} True if valid
     */
    isValidConfig(config) {
        return config && 
               typeof config.name === 'string' && 
               config.name.length > 0 &&
               typeof config.port === 'number' && 
               config.port > 0;
    }

    /**
     * List all configuration keys
     * @returns {string[]} Array of configuration keys
     */
    listConfigs() {
        return Array.from(this.configs.keys());
    }

    /**
     * Export configurations as JSON
     * @returns {string} JSON string
     */
    toJSON() {
        const data = {};
        for (const [key, config] of this.configs) {
            data[key] = config;
        }
        return JSON.stringify(data, null, 2);
    }
}

// Async function with modern syntax
async function loadConfigFromFile(filename) {
    try {
        const data = await fs.promises.readFile(filename, 'utf8');
        return JSON.parse(data);
    } catch (error) {
        console.error(`Failed to load config from ${filename}:`, error.message);
        return null;
    }
}

// Main function with various JavaScript features
async function main() {
    const manager = new ConfigManager();
    
    // Array of configurations with destructuring
    const configs = [
        { key: 'web', config: { name: 'webserver', port: 8080, enabled: true } },
        { key: 'api', config: { name: 'apiserver', port: 9000, enabled: false } },
        { key: 'db', config: { name: 'database', port: 5432, enabled: true } }
    ];
    
    // Process configurations with modern syntax
    for (const { key, config } of configs) {
        try {
            manager.addConfig(key, config);
        } catch (error) {
            console.error(`Error adding ${key}:`, error.message);
        }
    }
    
    // Display all configurations
    console.log('\nAll configurations:');
    manager.listConfigs().forEach(key => {
        const config = manager.getConfig(key);
        if (config) {
            const status = config.enabled ? 'enabled' : 'disabled';
            console.log(`  ${key}: ${config.name}:${config.port} (${status})`);
        }
    });
    
    // Performance measurement
    const start = performance.now();
    const summary = {
        timestamp: new Date().toISOString(),
        totalConfigs: manager.configs.size,
        activeConfigs: manager.listConfigs()
            .map(key => manager.getConfig(key))
            .filter(config => config && config.enabled)
            .length
    };
    const end = performance.now();
    
    console.log('\nSummary:', summary);
    console.log(`Processing time: ${(end - start).toFixed(2)}ms`);
    
    // Template literal example
    const report = `
Configuration Report
===================
Total: ${summary.totalConfigs}
Active: ${summary.activeConfigs}
Generated: ${summary.timestamp}
`;
    
    console.log(report);
}

// Error handling and execution
if (require.main === module) {
    main().catch(error => {
        console.error('Application error:', error);
        process.exit(1);
    });
}