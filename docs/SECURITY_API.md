# Security Module API Documentation

## Overview

The security module (`src/security.rs`) provides comprehensive protection against common file system vulnerabilities and security threats. This module is the foundation of Saorsa Browser's security architecture.

## Core Functions

### `validate_path(path: &Path, base_dir: &Path) -> Result<PathBuf, SecurityError>`

Validates that a path is safe and within allowed boundaries. This is the primary defense against path traversal attacks.

**Parameters:**
- `path: &Path` - The path to validate (should be relative)
- `base_dir: &Path` - The base directory that constrains access

**Returns:**
- `Ok(PathBuf)` - Canonicalized safe path within base directory
- `Err(SecurityError)` - Validation failure with specific error type

**Security Checks:**
1. **Null Byte Detection**: Prevents null byte injection attacks
2. **Absolute Path Rejection**: Only relative paths allowed
3. **Traversal Pattern Detection**: Blocks `../` and `..\` patterns
4. **Component Analysis**: Validates each path component individually
5. **Hidden File Protection**: Rejects paths starting with `.`
6. **Boundary Enforcement**: Ensures resolved path stays within base directory

**Example:**
```rust
use std::path::Path;
use crate::security::validate_path;

let base = Path::new("/safe/directory");
let user_input = Path::new("documents/file.md");

match validate_path(user_input, base) {
    Ok(safe_path) => {
        // Use safe_path for file operations
        println!("Safe path: {}", safe_path.display());
    }
    Err(e) => {
        // Log security event and reject request
        eprintln!("Security violation: {}", e);
    }
}
```

**Vulnerability Prevention:**
- **Path Traversal**: Prevents `../../../etc/passwd` style attacks
- **Null Byte Injection**: Blocks attempts to bypass file extension checks
- **Hidden File Access**: Protects system files starting with `.`
- **Symlink Attacks**: Canonicalization resolves symlinks safely

### `check_file_size(path: &Path) -> Result<u64, SecurityError>`

Validates file size against security limits to prevent resource exhaustion attacks.

**Parameters:**
- `path: &Path` - Path to the file to check

**Returns:**
- `Ok(u64)` - File size in bytes if within limits
- `Err(SecurityError::FileTooLarge(u64))` - File exceeds maximum allowed size

**Security Limits:**
- `MAX_FILE_SIZE`: 10 MB (10,485,760 bytes)
- `MAX_PREVIEW_SIZE`: 1 MB (1,048,576 bytes)

**Example:**
```rust
use crate::security::check_file_size;

match check_file_size(&file_path) {
    Ok(size) => {
        println!("File size OK: {} bytes", size);
        // Proceed with file operation
    }
    Err(SecurityError::FileTooLarge(size)) => {
        println!("File too large: {} bytes", size);
        // Reject or stream the file
    }
    Err(e) => {
        eprintln!("File check error: {}", e);
    }
}
```

**Resource Protection:**
- **Memory Exhaustion**: Prevents loading massive files into memory
- **DoS Prevention**: Limits processing of extremely large files
- **Performance**: Maintains responsive UI by avoiding large operations

### `sanitize_error_message(error: &str, _path: &Path) -> String`

Removes sensitive information from error messages before displaying to users or logging.

**Parameters:**
- `error: &str` - Original error message
- `_path: &Path` - Associated path (reserved for future use)

**Returns:**
- `String` - Sanitized error message with sensitive data removed

**Sanitization Patterns:**
- **File Paths**: Unix and Windows paths → `"file"`
- **IP Addresses**: `192.168.1.1` → `"[IP]"`
- **Credentials**: `password=secret` → `"password=[REDACTED]"`
- **Database URLs**: `postgresql://user:pass@host/db` → `"postgresql://[REDACTED]"`
- **Environment Variables**: `SECRET_KEY=value` → `"[ENV_VAR]=[REDACTED]"`

**Example:**
```rust
use crate::security::sanitize_error_message;

let error = "Failed to access /home/user/.secrets/password.txt";
let sanitized = sanitize_error_message(error, Path::new(""));
// Result: "Failed to access file"
```

**Information Protection:**
- **Path Disclosure**: Prevents leaking system structure
- **Credential Exposure**: Removes passwords and tokens
- **System Information**: Hides internal configuration details

## Error Types

### `SecurityError`

Comprehensive error enum covering all security-related failures.

```rust
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Path contains directory traversal")]
    PathTraversal,
    
    #[error("File too large: {0} bytes")]
    FileTooLarge(u64),
    
    #[error("Access denied")]
    AccessDenied,
    
    #[error("Invalid path")]
    InvalidPath,
    
    #[error("Path contains null byte")]
    NullByte,
    
    #[error("Path is hidden")]
    HiddenPath,
    
    #[error("Path is absolute")]
    AbsolutePath,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

**Error Categories:**
- **Input Validation**: `NullByte`, `InvalidPath`, `AbsolutePath`
- **Access Control**: `PathTraversal`, `HiddenPath`, `AccessDenied`
- **Resource Limits**: `FileTooLarge`
- **System Errors**: `IoError`

## Integration with Logging

The security module integrates with the structured logging system to provide comprehensive audit trails:

```rust
use tracing::{info, warn, error};

// Log security events with context
warn!("Path validation failed: {} for path: {}", 
      error, path.display());

// Track security metrics
info!("File size validation passed: {} bytes", size);
```

## Testing

The security module has comprehensive test coverage including:

### Unit Tests
- Path validation edge cases
- File size limit enforcement
- Error message sanitization
- Component-level validation

### Integration Tests
- End-to-end security workflows
- Attack scenario prevention
- Performance under load

### Property-Based Tests
- Fuzzing with random inputs
- Edge case discovery
- Invariant verification

## Best Practices

### For Developers

1. **Always Validate Inputs**: Use `validate_path` for all user-provided paths
2. **Check File Sizes**: Use `check_file_size` before processing files
3. **Sanitize Error Messages**: Use `sanitize_error_message` for user-facing errors
4. **Log Security Events**: Include security context in all logs
5. **Fail Securely**: Default to denial on validation failures

### For Operators

1. **Monitor Logs**: Watch for repeated security events
2. **Set Appropriate Limits**: Adjust file size limits based on use case
3. **Regular Updates**: Keep security dependencies updated
4. **Access Controls**: Implement appropriate base directory restrictions

## Security Considerations

### Threat Model

The security module protects against:
- **Malicious Users**: Attempting path traversal attacks
- **Resource Exhaustion**: DoS via large file uploads
- **Information Disclosure**: Leaking sensitive paths or data
- **Privilege Escalation**: Accessing restricted system files

### Limitations

- **Performance Impact**: Security checks add processing overhead
- **Usability Trade-offs**: Strict validation may reject legitimate paths
- **Platform Differences**: Path handling varies between OS platforms

### Future Enhancements

- **Content Validation**: Scan file contents for malicious patterns
- **Rate Limiting**: Prevent rapid-fire security violations
- **Advanced Logging**: More detailed security event classification
- **Configuration**: Runtime security policy configuration