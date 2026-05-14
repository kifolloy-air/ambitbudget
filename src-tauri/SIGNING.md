# Signing the Windows installers

Tauri's bundler is wired to call `sign.ps1` for every Windows artifact it
produces (the app `.exe`, the NSIS installer, and the MSI). The script signs
with the Sectigo OV code signing certificate held on the SafeNet eToken Fusion
FIPS USB token.

Signing is **opt-in**. The script only runs `signtool` when the environment
variable `AMBIT_SIGN=1` is set. Without that variable, the script exits 0 and
the build produces unsigned installers — which is exactly what the
GitHub-hosted CI runner needs, since it has no token attached.

## Certificate at a glance

| Field | Value |
|---|---|
| Subject | `CN="Kifolloy AIR, LLC", O="Kifolloy AIR, LLC", S=Pennsylvania, C=US` |
| Issuer | Sectigo Public Code Signing CA R36 |
| SHA-1 thumbprint | `583F11C19B8F6C0BC04CB304BE537EA525E59EF8` |
| Timestamp server | `http://timestamp.sectigo.com` |
| Valid through | **2027-05-06** (renew by April 2027) |
| Hardware | SafeNet eToken Fusion FIPS USB token |
| Cert store path | `Cert:\CurrentUser\My` (`HasPrivateKey: True`) |

The thumbprint, timestamp server, and `signtool.exe` lookup live in `sign.ps1`.
If any of those change (e.g. you renew the cert and get a new thumbprint, or
install a newer Windows SDK), edit `sign.ps1` directly.

## Build a signed installer locally

1. **Plug in the SafeNet eToken.** Wait for Windows to recognize it. SafeNet
   Authentication Client should be installed and running.

2. **Open PowerShell** in the `website/src-tauri/` directory.

3. **Turn on signing for this shell:**

   ```powershell
   $env:AMBIT_SIGN = "1"
   ```

   This sets the variable for the current PowerShell session only. Close the
   shell and the variable is gone, which is the safe default — you won't
   accidentally trigger a token prompt on an unrelated build.

   If you want signing on for *every* shell, set it permanently:

   ```powershell
   [Environment]::SetEnvironmentVariable("AMBIT_SIGN", "1", "User")
   ```

   (You'd then need to open a fresh shell for the variable to take effect.)

4. **Build:**

   ```powershell
   cargo tauri build
   ```

   The build runs as usual. Near the end, for each artifact, SafeNet
   Authentication Client will **pop up a dialog asking for your Token
   Password**. Type the PIN into the SafeNet dialog. **Do not type your PIN
   into any AI assistant, terminal, or chat window.** Only the SafeNet GUI
   prompt is legitimate.

   You'll be prompted three times by default (main `.exe`, NSIS installer,
   MSI installer). SafeNet may offer a "remember for this session" checkbox —
   ticking it reduces the number of prompts within a single build.

5. **Verify the signatures** on the produced installers:

   ```powershell
   $sdk = (Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.*.*.*\x64\signtool.exe" |
           Sort-Object FullName -Descending | Select-Object -First 1).FullName
   & $sdk verify /pa /v "target\release\bundle\nsis\Ambit Budget Pro_*_x64-setup.exe"
   & $sdk verify /pa /v "target\release\bundle\msi\Ambit Budget Pro_*_x64_en-US.msi"
   ```

   Expect `Successfully verified` and a chain ending at the Sectigo root.

## Disabling signing

Just don't set `AMBIT_SIGN`. The wrapper script no-ops and the build produces
unsigned installers. CI does this every push to `main`.

If you want to disable signing more permanently (e.g. you've lost the token
and need to ship unsigned builds), you can also delete the `signCommand`
block from `tauri.conf.json`. Tauri will then skip the wrapper entirely.

## If the SafeNet PIN dialog locks (5-strike rule)

The SafeNet eToken Fusion FIPS enforces strict lockout limits:

- **5 wrong Token Password (user PIN) attempts → the user PIN is locked.**
  The token will no longer sign anything. The certificate is still on the
  token, but you can't access it with the user PIN.
- **5 wrong Administrator Password attempts AFTER the user PIN is locked →
  the token is permanently bricked.** It cannot be recovered. You would have
  to re-enroll a new certificate on a new token (≈ Sectigo will need to
  re-issue, which is the cost of a new OV cert).

**If you get a "Token Password is locked" error:**

1. **Stop. Do not start guessing the admin password.** Once the user PIN is
   locked, every wrong admin attempt brings you closer to a brick.
2. Open **SafeNet Authentication Client Tools** (search for it in Start).
3. Right-click the token in the left pane → **Set Token Password**.
4. SafeNet will ask for the **Administrator Password** (the one set when the
   token was first initialized — usually documented separately from the
   Token Password, kept in your password manager).
5. With the correct admin password, you can unlock and set a new Token
   Password. The admin password unlock counter resets when you succeed.

If you do not have the Administrator Password recorded anywhere, **do not
attempt to guess it.** Contact Sectigo support before doing anything else —
they may be able to help re-issue. Guessing puts the token at risk of
permanent destruction.

The Token Password and Admin Password should be stored in a password manager
(e.g. 1Password, Bitwarden). If they are not, store them there now.

## Renewal — April 2027 reminder

The certificate expires **2027-05-06**. Sectigo OV code signing certificates
are issued for one year. Plan to renew **by early April 2027** so there's
buffer for:

- Sectigo's vetting and re-issuance turnaround (usually 1–3 business days for
  renewals).
- A possible second hardware token if the cert is issued onto a new device.
- Updating `sign.ps1` with the new SHA-1 thumbprint after renewal — the
  thumbprint changes whenever a new certificate is issued, even on the same
  legal entity.

When you renew:

1. Check the new cert's SHA-1 thumbprint:

   ```powershell
   Get-ChildItem Cert:\CurrentUser\My |
     Where-Object { $_.Subject -like "*Kifolloy AIR*" } |
     Select-Object Thumbprint, NotAfter, Subject
   ```

2. Update the `$thumbprint` value in `sign.ps1`.
3. Run one signed build and verify (`signtool verify /pa /v ...`) before
   pushing the change.

## How the wiring works (for future maintenance)

`tauri.conf.json` → `bundle.windows.signCommand`:

```json
{
  "cmd": "powershell",
  "args": [
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", "sign.ps1",
    "-Target", "%1"
  ]
}
```

Tauri 2 substitutes `%1` with the absolute path to each binary as it bundles.
It also resolves the relative `sign.ps1` argument to an absolute path using
the directory of `tauri.conf.json`, so the call works regardless of where
`cargo tauri build` is invoked from.

The PowerShell wrapper:

1. Exits 0 immediately if `AMBIT_SIGN != "1"` (CI path).
2. Locates `signtool.exe` under the Windows SDK — preferring
   `10.0.26100.0`, falling back to the highest installed SDK version.
3. Runs `signtool sign /sha1 <thumb> /fd SHA256 /tr <timestamp> /td SHA256 /v <target>`.
4. Propagates `signtool`'s exit code so a bad signature fails the build.

No secrets are stored in the repo. The private key never leaves the
SafeNet token.
