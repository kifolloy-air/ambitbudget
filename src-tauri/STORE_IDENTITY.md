# Microsoft Store — Package Identity

Captured 2026-05-10 from Partner Center → Apps and games → Ambit Budget Pro → Product Identity.

These values **must** be embedded in any future MSIX build that's intended as an
update to the existing live store listing. A mismatched Publisher CN or Identity
Name causes the Store to treat the upload as a different app, breaks updates for
existing installs, and likely fails certification.

## Identity (matches store-assigned values)

| Field | Value |
|---|---|
| `Package/Identity/Name` | `AmbitBudgetPro.AmbitBudgetPro` |
| `Package/Identity/Publisher` | `CN=9B4717E7-8B65-43B9-81D1-57ADF6460F30` |
| `Package/Properties/PublisherDisplayName` | `Ambit Budget Pro` |
| Package Family Name (PFN) | `AmbitBudgetPro.AmbitBudgetPro_vqzafymjp1zbr` |
| Store ID | `9P4V21SHBK8X` |
| Store URL | https://apps.microsoft.com/detail/9P4V21SHBK8X |

## Current submission

- **Submission 1**, last modified 2026-03-30
- Type: **MSIX or PWA app** (PWA wrapper from PWABuilder)
- Status: **In the Microsoft Store** (live, 41 markets)
- Cert report: 2026-03-30 08:18 AM (passed)

## Future MSIX build notes

Tauri 2 doesn't have a first-party MSIX bundle target as of this writing. Paths
forward when we want to switch the store package from PWA wrapper → native
Tauri MSIX:

1. **MSIX Packaging Tool** (Microsoft, free) — wrap our NSIS installer into
   MSIX. Manual, but supported.
2. **MakeAppx.exe** (Windows SDK) — build the AppxManifest.xml ourselves and
   pack the Tauri output. Most control, most work.
3. **cargo-packager** — alternative bundler that supports MSIX. Replacing
   Tauri's bundler for the Windows target.
4. **Wait for Tauri to ship MSIX target** (active community feature request).

Whatever path: the AppxManifest.xml MUST contain:

```xml
<Identity Name="AmbitBudgetPro.AmbitBudgetPro"
          Publisher="CN=9B4717E7-8B65-43B9-81D1-57ADF6460F30"
          Version="1.0.1.0" />
<Properties>
  <DisplayName>Ambit Budget Pro</DisplayName>
  <PublisherDisplayName>Ambit Budget Pro</PublisherDisplayName>
</Properties>
```

`Version` is bumped per submission. The 4-part scheme (W.X.Y.Z) is required;
the last part must always be 0.

## Direct distribution alternative

For users who want the real desktop install today (filesystem storage,
better OS integration), the Tauri NSIS .exe and MSI .msi are built by
GitHub Actions on every push to main. Download from the latest run's
artifacts at https://github.com/kifolloy-air/ambitbudget/actions or
attach to a GitHub Release. The website should expose a "Download for
Windows" link to either of those.
