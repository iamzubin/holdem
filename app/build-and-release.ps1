# Parameters
$version = "0.2.0"
# $token = "token"  # GitHub token should be set as an environment variable
$owner = "iamzubin"  # Your GitHub username
$repo = "holdem"      # Repository name

# Install dependencies
Write-Host "Installing dependencies..."
pnpm install

# Build the application
Write-Host "Building application..."
pnpm tauri build

# Get the installer and signature files
$installerFile = Get-ChildItem -Path "src-tauri/target/release/bundle/nsis" -Filter "holdem_$version`_x64-setup.exe" | Select-Object -First 1
$signatureFile = Get-ChildItem -Path "src-tauri/target/release/bundle/nsis" -Filter "*.sig" | Select-Object -First 1
$binaryFile = Get-ChildItem -Path "src-tauri/target/release" -Filter "*.exe" | Select-Object -First 1

if (-not $installerFile -or -not $signatureFile -or -not $binaryFile) {
    Write-Host "Error: Could not find required files"
    Write-Host "Installer: $($installerFile.FullName)"
    Write-Host "Signature: $($signatureFile.FullName)"
    Write-Host "Binary: $($binaryFile.FullName)"
    exit 1
}

# Read the signature
$signature = Get-Content $signatureFile.FullName -Raw

# Create update.json content
$updateJson = @{
    version = $version
    notes = "Release notes for Holdem $version"
    pub_date = [DateTime]::UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = $signature.Trim()
            url = "https://github.com/$owner/$repo/releases/download/$version/holdem_$version`_x64-setup.exe"
        }
    }
} | ConvertTo-Json -Depth 10

# Save update.json without BOM in the holdem_website/public directory
$updateJsonPath = Join-Path $PSScriptRoot "../holdem_website/public/update.json"
[System.IO.File]::WriteAllText($updateJsonPath, $updateJson, [System.Text.Encoding]::UTF8)

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
    Write-Host "Installer path: $($installerFile.FullName)"
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
    Write-Host "Installer path: $($installerFile.FullName)"
}

# Upload the binary
try {
    $uploadUrl = $release.upload_url -replace "\{\?name,label\}", "?name=$($binaryFile.Name)"
    Write-Host "Uploading binary..."
    $fileBytes = [System.IO.File]::ReadAllBytes($binaryFile.FullName)
    $uploadHeaders = @{
        "Accept" = "application/vnd.github+json"
        "Authorization" = "Bearer $token"
        "X-GitHub-Api-Version" = "2022-11-28"
        "Content-Type" = "application/octet-stream"
    }
    Invoke-RestMethod -Uri $uploadUrl -Method Post -Headers $uploadHeaders -Body $fileBytes
    Write-Host "Successfully uploaded binary"
} catch {
    Write-Host "Error uploading binary: $_"
    Write-Host "Response: $($_.ErrorDetails.Message)"
}

# Upload the NSIS installer
try {
    $uploadUrl = $release.upload_url -replace "\{\?name,label\}", "?name=$($installerFile.Name)"
    Write-Host "Uploading NSIS installer..."
    $fileBytes = [System.IO.File]::ReadAllBytes($installerFile.FullName)
    $uploadHeaders = @{
        "Accept" = "application/vnd.github+json"
        "Authorization" = "Bearer $token"
        "X-GitHub-Api-Version" = "2022-11-28"
        "Content-Type" = "application/octet-stream"
    }
    Invoke-RestMethod -Uri $uploadUrl -Method Post -Headers $uploadHeaders -Body $fileBytes
    Write-Host "Successfully uploaded NSIS installer"
} catch {
    Write-Host "Error uploading NSIS installer: $_"
    Write-Host "Response: $($_.ErrorDetails.Message)"
}

# Update the updater.json URL in tauri.conf.json to use raw content
$tauriConfPath = Join-Path $PSScriptRoot "src-tauri/tauri.conf.json"
$tauriConf = Get-Content $tauriConfPath -Raw | ConvertFrom-Json
$tauriConf.plugins.updater.endpoints = @("https://raw.githubusercontent.com/$owner/$repo/main/updater.json")
$tauriConf | ConvertTo-Json -Depth 10 | Set-Content $tauriConfPath

Write-Host "Release process completed!" 