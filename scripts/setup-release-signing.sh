#!/usr/bin/env bash
# One-time setup for GPG-signed release checksums.
# Writes the PUBLIC key to docs/release-signing.pub.asc (safe to commit).
# Sends the PRIVATE key to GitHub Actions secrets only — never to the repo.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PUBLIC_KEY="$ROOT/docs/release-signing.pub.asc"
GNUPGHOME="$(mktemp -d)"
export GNUPGHOME
PASSPHRASE="${FGC_GPG_PASSPHRASE:-$(openssl rand -base64 24)}"
KEY_EMAIL="${FGC_GPG_EMAIL:-fgc-scoreboard-releases@users.noreply.github.com}"

cleanup() { rm -rf "$GNUPGHOME"; }
trap cleanup EXIT

if ! command -v gpg >/dev/null 2>&1; then
	echo "gpg not found. Install GnuPG and re-run." >&2
	exit 1
fi

assert_public_key() {
	local file="$1"
	if grep -q 'BEGIN PGP PRIVATE KEY BLOCK' "$file"; then
		echo "Refusing to write private key material to $file" >&2
		exit 1
	fi
	if ! grep -q 'BEGIN PGP PUBLIC KEY BLOCK' "$file"; then
		echo "Expected a public key block in $file" >&2
		exit 1
	fi
}

cat > "$GNUPGHOME/gpg-batch.conf" <<EOF
%no-protection
Key-Type: RSA
Key-Length: 4096
Key-Usage: sign
Name-Real: FGC Scoreboard Release
Name-Email: ${KEY_EMAIL}
Expire-Date: 0
EOF

gpg-agent --daemon --homedir "$GNUPGHOME" >/dev/null 2>&1 || true
gpg --batch --homedir "$GNUPGHOME" --generate-key "$GNUPGHOME/gpg-batch.conf"

KEY_ID="$(gpg --homedir "$GNUPGHOME" --list-secret-keys --with-colons | awk -F: '/^sec:/ { print $5; exit }')"
gpg --homedir "$GNUPGHOME" --armor --export "$KEY_ID" > "$PUBLIC_KEY"
assert_public_key "$PUBLIC_KEY"

echo "Wrote $PUBLIC_KEY (public key $KEY_ID, $KEY_EMAIL)"

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
	PRIVATE_KEY="$(gpg --homedir "$GNUPGHOME" --armor --export-secret-keys "$KEY_ID")"
	if ! grep -q 'BEGIN PGP PRIVATE KEY BLOCK' <<< "$PRIVATE_KEY"; then
		echo "Failed to export private key for GitHub secrets" >&2
		exit 1
	fi
	gh secret set GPG_PRIVATE_KEY --body "$PRIVATE_KEY"
	gh secret set GPG_PASSPHRASE --body "$PASSPHRASE"
	echo "Set GitHub secrets GPG_PRIVATE_KEY and GPG_PASSPHRASE (private key is NOT written to disk)."
else
	echo "gh not authenticated — add secrets manually (see docs/signing.md)."
	echo "Never commit output from: gpg --armor --export-secret-keys $KEY_ID"
fi

echo "Commit docs/release-signing.pub.asc only, then tag a release."
