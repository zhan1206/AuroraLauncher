# Security Policy

## Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.x.x   | ✅ Active development |

## Reporting a Vulnerability

请通过 GitHub Issues 或邮件报告安全漏洞。请勿公开披露，等待修复后再公告。

## Token Storage

Aurora Launcher 使用操作系统密钥链（Windows Credential Manager / macOS Keychain / Linux Secret Service）存储 Microsoft OAuth Token，不以明文写入磁盘。
