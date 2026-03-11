# Security Policy

## Supported versions

| Version | Supported |
| --- | --- |
| 7.0.x | Yes |
| < 7.0 | No |

## Reporting a vulnerability

Please report security issues **privately** through
[GitHub Security Advisories](https://github.com/katara-project/katara/security/advisories/new).

Do **not** open a public issue for security vulnerabilities.

We aim to acknowledge reports within 48 hours and provide a fix or mitigation
within 7 days for critical issues.

## Security design goals

- **Minimal context exposure:** only the compiled context reaches the LLM.
- **Policy-driven routing:** sensitive data stays on local providers.
- **Sovereign-first defaults:** on-prem deployments are first-class citizens.
- **Explicit sensitive handling:** context blocks are tagged by sensitivity level.

## Current scaffold limitations

The following are planned but not yet implemented:

- Full secret scanning enforcement
- Production authentication and authorization
- Encrypted memory block persistence
- Hardened provider credential rotation
- TLS enforcement on inter-service communication
