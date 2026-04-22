# Security Policy

## Supported Versions

Security fixes are applied to the current development line only.

There is no promise of backport support for older tags or unpublished local states.

## Reporting a Vulnerability

Do not open a public issue for a credential leak, auth flaw, or other security-sensitive bug.

Instead:

1. Report it privately to `contact@micr.dev`.
2. Include a minimal reproduction and impact summary.
3. State whether the issue affects GitHub.com only, custom hosts, or both.
4. Avoid including real tokens in the report body.

## What to Include

Useful reports usually include:

- affected command or code path
- expected behavior
- actual behavior
- steps to reproduce
- impact assessment
- whether a workaround exists

## Credentials and Secrets

Never commit:

- PATs
- environment files containing live tokens
- captured credential-store contents
- screenshots or logs containing real credentials

If you accidentally expose a token while testing, rotate it immediately.
