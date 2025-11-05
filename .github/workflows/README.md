# GitHub Actions Workflows

This directory contains CI/CD workflows for TidyFiles.

## Workflows

### 1. Test & Build Check (`test.yml`)

**Trigger**: Runs on every push to `main` and on pull requests

**Purpose**: Ensures code quality and build integrity

**Steps:**
- Runs Rust checks (`cargo check`, `cargo clippy`, `cargo fmt`)
- Runs Rust tests
- Builds frontend (Svelte)
- Builds Tauri app in debug mode
- Uploads debug builds as artifacts (available for 7 days)

**Usage**: Automatic - no action needed

---

### 2. Release Windows Installer (`release.yml`)

**Trigger**: Runs when you push a version tag (e.g., `v0.1.0`, `v1.2.3`)

**Purpose**: Creates production releases with installers

**Steps:**
- Builds production Tauri app (optimized)
- Creates GitHub Release
- Uploads MSI and NSIS installers
- Keeps artifacts for 90 days

**Usage:**

1. **Update version** in `src-tauri/tauri.conf.json`:
   ```json
   {
     "version": "0.2.0"
   }
   ```

2. **Commit changes**:
   ```bash
   git add .
   git commit -m "Release v0.2.0"
   ```

3. **Create and push tag**:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **GitHub Actions will automatically**:
   - Build Windows installers
   - Create a new release at `https://github.com/wihlarkop/tidyfiles/releases`
   - Upload installers to the release

---

## Code Signing (Optional but Recommended)

To avoid Windows SmartScreen warnings, you can add code signing:

### Requirements:
- Code signing certificate (OV or EV)
- Certificate stored as GitHub Secret

### Setup:

1. **Get a code signing certificate**:
   - **EV Certificate** (recommended): No SmartScreen warnings immediately
   - **OV Certificate** (cheaper): SmartScreen warnings initially, builds reputation over time

2. **Add certificate to GitHub Secrets**:
   - Go to: `Settings` → `Secrets and variables` → `Actions`
   - Add new secrets:
     - `WINDOWS_CERTIFICATE`: Base64-encoded .pfx file
     - `WINDOWS_CERTIFICATE_PASSWORD`: Certificate password

3. **Uncomment code signing section** in `release.yml`:
   ```yaml
   - name: Setup code signing
     run: |
       echo "${{ secrets.WINDOWS_CERTIFICATE }}" | base64 --decode > cert.pfx
       # Add certificate to Windows certificate store
   ```

4. **Update tauri.conf.json** with signing config:
   ```json
   {
     "bundle": {
       "windows": {
         "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
         "digestAlgorithm": "sha256",
         "timestampUrl": "http://timestamp.comodoca.com"
       }
     }
   }
   ```

### Certificate Providers:
- [DigiCert](https://www.digicert.com/signing/code-signing-certificates)
- [Sectigo (formerly Comodo)](https://sectigo.com/ssl-certificates-tls/code-signing)
- [SSL.com](https://www.ssl.com/certificates/code-signing/)

---

## Monitoring Workflows

1. **View workflow runs**:
   - Go to: `Actions` tab in GitHub repository
   - Click on workflow name to see runs

2. **Check build status**:
   - Green checkmark = success
   - Red X = failure (click to see logs)

3. **Download artifacts**:
   - Click on successful workflow run
   - Scroll to "Artifacts" section
   - Download installers

---

## Troubleshooting

### Build fails with "No such file or directory"
- Check that all paths in workflow match your project structure
- Ensure `src-tauri/tauri.conf.json` exists and is valid

### Release workflow doesn't trigger
- Verify tag format: must be `vX.Y.Z` (e.g., `v0.1.0`)
- Check that tag was pushed: `git push origin --tags`

### Code signing fails
- Verify certificate secrets are set correctly in GitHub
- Check certificate is valid and not expired
- Ensure certificate thumbprint matches in config

---

## Future Enhancements

When ready to expand beyond Windows:

1. Add macOS workflow (requires macOS runner)
2. Add Linux workflow (can run on ubuntu-latest)
3. Implement auto-updater integration
4. Add automated testing (unit tests, integration tests)

---

## Resources

- [Tauri Distribution Guide](https://v2.tauri.app/distribute/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Tauri Code Signing](https://v2.tauri.app/distribute/sign/windows/)
