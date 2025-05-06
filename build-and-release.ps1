# Parameters
$version = "0.1.2"
# $token = "token"  # GitHub token should be set as an environment variable
$owner = "iamzubin"  # Your GitHub username
$repo = "holdem"      # Repository name

# # Install dependencies
# Write-Host "Installing dependencies..."
# pnpm install

# # Build the application
# Write-Host "Building application..."
# pnpm tauri build

# Get the installer and signature files
$installerFile = Get-ChildItem -Path "src-tauri/target/release/bundle/nsis" -Filter "*setup.exe"
$signatureFile = Get-ChildItem -Path "src-tauri/target/release/bundle/nsis" -Filter "*.sig"

# Read the signature
$signature = Get-Content $signatureFile.FullName -Raw

# Create updater.json content
$updaterJson = @{
    version = $version
    notes = "Release notes for Holdem $version"
    pub_date = [DateTime]::UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = $signature.Trim()
            url = "https://github.com/$owner/$repo/releases/download/$version/$($installerFile.Name)"
        }
    }
} | ConvertTo-Json -Depth 10

# Save updater.json without BOM
$updaterJsonPath = Join-Path $PSScriptRoot "updater.json"
[System.IO.File]::WriteAllText($updaterJsonPath, $updaterJson, [System.Text.Encoding]::UTF8)

# Create the release
$releaseData = @{
    tag_name = $version
    target_commitish = "main"  # or your default branch
    name = "Holdem $version"
    body = "Release notes for Holdem $version"
    draft = $true
    prerelease = $false
} | ConvertTo-Json

$headers = @{
    "Accept" = "application/vnd.github+json"
    "Authorization" = "Bearer $token"
    "X-GitHub-Api-Version" = "2022-11-28"
}

# Create the release
Write-Host "Creating GitHub release..."
try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$owner/$repo/releases" -Method Post -Headers $headers -Body $releaseData
    Write-Host "Release created successfully!"
} catch {
    Write-Host "Error creating release: $_"
    Write-Host "Response: $($_.ErrorDetails.Message)"
    exit 1
}

# Upload the installer
try {
    $uploadUrl = $release.upload_url -replace "\{\?name,label\}", "?name=$($installerFile.Name)"
    Write-Host "Uploading installer..."
    $fileBytes = [System.IO.File]::ReadAllBytes($installerFile.FullName)
    $uploadHeaders = @{
        "Accept" = "application/vnd.github+json"
        "Authorization" = "Bearer $token"
        "X-GitHub-Api-Version" = "2022-11-28"
        "Content-Type" = "application/octet-stream"
    }
    Invoke-RestMethod -Uri $uploadUrl -Method Post -Headers $uploadHeaders -Body $fileBytes
    Write-Host "Successfully uploaded installer"
} catch {
    Write-Host "Error uploading installer: $_"
    Write-Host "Response: $($_.ErrorDetails.Message)"
}

# Upload updater.json
try {
    $uploadUrl = $release.upload_url -replace "\{\?name,label\}", "?name=updater.json"
    Write-Host "Uploading updater.json..."
    $fileBytes = [System.IO.File]::ReadAllBytes($updaterJsonPath)
    $uploadHeaders = @{
        "Accept" = "application/vnd.github+json"
        "Authorization" = "Bearer $token"
        "X-GitHub-Api-Version" = "2022-11-28"
        "Content-Type" = "application/json"
    }
    Invoke-RestMethod -Uri $uploadUrl -Method Post -Headers $uploadHeaders -Body $fileBytes
    Write-Host "Successfully uploaded updater.json"
} catch {
    Write-Host "Error uploading updater.json: $_"
    Write-Host "Response: $($_.ErrorDetails.Message)"
}

# Update the updater.json URL in tauri.conf.json to use raw content
$tauriConfPath = Join-Path $PSScriptRoot "src-tauri/tauri.conf.json"
$tauriConf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
$tauriConf.plugins.updater.endpoints = @("https://raw.githubusercontent.com/$owner/$repo/main/updater.json")
$tauriConf | ConvertTo-Json -Depth 10 | Set-Content $tauriConfPath

Write-Host "Release process completed!" 