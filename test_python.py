#!/usr/bin/env python3
"""
A sample Python script to test syntax highlighting
"""

import json
import os
from dataclasses import dataclass
from datetime import datetime
from typing import Dict, List, Optional, Union


@dataclass
class Config:
    """Configuration data class"""

    name: str
    port: int
    enabled: bool = True

    def is_valid(self) -> bool:
        """Check if configuration is valid"""
        return len(self.name) > 0 and self.port > 0


class ServerManager:
    """Manages server configurations"""

    def __init__(self):
        self.configs: Dict[str, Config] = {}
        self._initialized = False

    def add_config(self, key: str, config: Config) -> None:
        """Add a configuration"""
        if not config.is_valid():
            raise ValueError(f"Invalid config for '{key}'")

        self.configs[key] = config
        print(f"Added config: {key} -> {config}")

    def get_config(self, key: str) -> Optional[Config]:
        """Get a configuration by key"""
        return self.configs.get(key)

    def list_configs(self) -> List[str]:
        """List all configuration keys"""
        return list(self.configs.keys())


def main():
    """Main function"""
    manager = ServerManager()

    # Create some configurations
    configs = [
        ("web", Config("webserver", 8080, True)),
        ("api", Config("apiserver", 9000, False)),
        ("db", Config("database", 5432)),
    ]

    for key, config in configs:
        try:
            manager.add_config(key, config)
        except ValueError as e:
            print(f"Error: {e}")

    # List and display configurations
    print("\nAll configurations:")
    for key in manager.list_configs():
        config = manager.get_config(key)
        if config:
            status = "enabled" if config.enabled else "disabled"
            print(f"  {key}: {config.name}:{config.port} ({status})")

    # JSON serialization example
    data = {
        "timestamp": datetime.now().isoformat(),
        "configs": len(manager.configs),
        "active": sum(1 for c in manager.configs.values() if c.enabled),
    }

    print(f"\nSummary: {json.dumps(data, indent=2)}")


if __name__ == "__main__":
    main()
