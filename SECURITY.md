# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| < 0.3   | :x:                |

## Reporting a Vulnerability

We take the security of Saorsa Browser seriously. If you discover a security vulnerability, please follow these steps:

1. **DO NOT** open a public issue
2. Email your findings to [security contact - please update with your email]
3. Include the following information:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if available)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 5 business days
- **Resolution Target**: Critical issues within 30 days

## Security Features

### Path Traversal Protection
- All file paths are validated against directory traversal attempts
- Canonical path resolution ensures files stay within allowed directories
- Hidden files and system files are blocked by default
- See `src/security.rs` for implementation details

### Resource Limits
- **File Size Limit**: 10MB for file operations
- **Preview Limit**: 1MB for rendered previews
- **Memory Protection**: Bounded buffers for all file operations

### Input Validation
- All user inputs are sanitized before processing
- Command injection protection in file operations
- Null byte filtering in paths

### Error Handling
- Sensitive information is redacted from error messages
- File paths, IP addresses, and credentials are sanitized
- Stack traces are filtered in production builds

## Security Best Practices

### For Contributors

1. **Dependencies**
   - Run `cargo audit` before submitting PRs
   - Keep dependencies up to date
   - Avoid unmaintained packages

2. **Code Review**
   - All PRs require security review
   - Use `#[forbid(unsafe_code)]` where possible
   - Document any unsafe blocks with justification

3. **Testing**
   - Include security tests for new features
   - Test boundary conditions and error cases
   - Use property-based testing for input validation

### For Users

1. **Installation**
   - Download only from official sources
   - Verify checksums when available
   - Keep the application updated

2. **Usage**
   - Avoid opening untrusted files
   - Be cautious with symbolic links
   - Report suspicious behavior immediately

## Known Security Considerations

### Dependencies
We are aware of the following unmaintained dependencies:

#### Low Risk - Acceptable 
- `paste` (via ratatui) - Proc-macro helper, no known vulnerabilities, core TUI framework dependency
- `yaml-rust` (via tui-markdown->syntect) - YAML parsing for syntax definitions, no known vulnerabilities

**Risk Assessment**: These dependencies pose minimal security risk as they are:
- Not directly handling user input or network data
- Used in well-contained contexts (UI rendering, syntax highlighting)
- Part of mature, widely-used libraries
- No known CVEs against current versions

**Mitigation**: Regular monitoring via `cargo audit` and security alerts. Alternative implementations would require significant architectural changes that don't justify the low risk level.

### External Commands
- **ffmpeg**: Used for video playback. Ensure you have the latest version installed
- **git**: Used for diff functionality. Keep Git updated to latest stable version

## Security Audit

Regular security audits are performed:
- **Automated**: Weekly via GitHub Actions (cargo-audit)
- **Manual**: Quarterly code review focusing on security
- **External**: Annual third-party security assessment (when applicable)

## Compliance

This project follows security best practices including:
- OWASP guidelines for application security
- Rust security guidelines from the Rust Security Response WG
- CWE Top 25 Most Dangerous Software Weaknesses mitigation

## Security Updates

Security updates are released as soon as fixes are available:
- **Critical**: Immediate patch release
- **High**: Within 7 days
- **Medium**: Within 30 days
- **Low**: Next regular release

## Contact

For security concerns, please contact:
- Primary: [Please add security contact email]
- Secondary: Open an issue with "Security" label (for non-sensitive discussions)

## Acknowledgments

We appreciate responsible disclosure and will acknowledge security researchers who:
- Follow responsible disclosure practices
- Allow reasonable time for patching
- Don't exploit vulnerabilities beyond proof of concept

Contributors will be credited in our security advisories unless they prefer to remain anonymous.