# Release signing (GPG)

Release archives are checksummed in CI. When `GPG_PRIVATE_KEY` is configured, the workflow also publishes a detached GPG signature for `SHA256SUMS.txt`.

## Public vs private key

| Material | Where it lives | Commit to git? |
|----------|----------------|----------------|
| **Public key** | [`docs/release-signing.pub.asc`](release-signing.pub.asc) | Yes — required for users to verify downloads |
| **Private key** | GitHub Actions secret `GPG_PRIVATE_KEY` | **Never** |
| **Passphrase** | GitHub Actions secret `GPG_PASSPHRASE` | **Never** |

Files containing `BEGIN PGP PRIVATE KEY BLOCK` must never be added to this public repo. `.gitignore` blocks common private-key filename patterns.

## Maintainer setup (one-time)

Run the setup script (requires GnuPG; uses `gh` to set repo secrets when authenticated):

```bash
./scripts/setup-release-signing.sh
```

This writes **`docs/release-signing.pub.asc`** (public key only) and sets `GPG_PRIVATE_KEY` / `GPG_PASSPHRASE` in GitHub Actions secrets. Commit the `.pub.asc` file, then tag a release.

Manual setup (alternative):

1. Generate a signing key:

   ```bash
   gpg --full-generate-key
   ```

   Ed25519 or RSA 4096 is fine. Use a dedicated release-signing identity (name/email you will keep long-term).

2. Publish the **public** key in the repo:

   ```bash
   gpg --armor --export YOUR_KEY_ID > docs/release-signing.pub.asc
   git add docs/release-signing.pub.asc
   ```

3. Send the **private** key to GitHub Actions only (do not save to a file in the repo):

   ```bash
   gpg --armor --export-secret-keys YOUR_KEY_ID
   # Copy output into Settings → Secrets → GPG_PRIVATE_KEY
   ```

4. Add repository secrets (**Settings → Secrets and variables → Actions**):

   | Secret | Value |
   |--------|-------|
   | `GPG_PRIVATE_KEY` | Full armored **private** key block from step 3 |
   | `GPG_PASSPHRASE` | Key passphrase (empty string if none) |

5. Tag a release as usual (`v*`). The [Release workflow](.github/workflows/release.yml) attaches `SHA256SUMS.txt` and, when secrets are set, `SHA256SUMS.txt.asc`.

If secrets are missing (e.g. forks), releases still ship with unsigned checksums only.

## Verifying a download

From the [GitHub Release](https://github.com/wukrit/fgc-scoreboard/releases) page, download the platform archive(s) you need plus `SHA256SUMS.txt`. If the release includes `SHA256SUMS.txt.asc`, verify the signature first:

```bash
gpg --import docs/release-signing.pub.asc
gpg --verify SHA256SUMS.txt.asc SHA256SUMS.txt
```

Then confirm archive hashes (run from the folder containing the downloaded files):

```bash
# Linux
sha256sum -c SHA256SUMS.txt

# macOS
shasum -a 256 -c SHA256SUMS.txt

# Windows (PowerShell)
Get-FileHash .\fgc-scoreboard-*-x86_64-pc-windows-msvc.zip -Algorithm SHA256
# Compare output to the matching line in SHA256SUMS.txt
```

GPG signing proves the checksum file is authentic. It does **not** remove macOS Gatekeeper or Windows SmartScreen warnings for unsigned binaries — see [README.md](../README.md#verifying-downloads).
