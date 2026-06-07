# Incident response

## Reporting

Email **security@vericonomy.com** — do not file public GitHub issues for vulnerabilities.

Include:

- Affected version and platform
- Steps to reproduce
- Impact assessment (confidentiality / integrity / availability)
- Proof of concept if available

## Intake timeline (target)

| Stage | Target |
| --- | --- |
| Acknowledgment | 2 business days |
| Triage severity | 5 business days |
| Fix or mitigation plan | 15 business days (critical/high) |
| Coordinated disclosure | After fix shipped or 90 days |

## Severity classes

| Class | Examples |
| --- | --- |
| Critical | Remote key extraction without user action, 2FA bypass on send |
| High | Plaintext secret persistence, unrestricted RPC in production |
| Medium | Information leak in logs, weak defaults |
| Low | UX security issues, documentation gaps |

## Hotfix process

1. Reproduce on `main` branch
2. Patch in `desktop/verium-app`
3. Run `cargo test`, `npm test`, `cargo audit`, `npm audit`
4. Tag `desktop-v*` patch release
5. Publish release notes with CVE reference if applicable
6. Notify reporters before public disclosure

## Contacts

- Security: security@vericonomy.com
- Maintainers: see repository `CODEOWNERS` / release workflow
