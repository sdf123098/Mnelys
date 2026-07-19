[CmdletBinding()]
param(
    [string]$Configuration = "Release",
    [string]$OutputRoot
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..")).Path
$project = Join-Path $repositoryRoot "src\Mnelys.Desktop\Mnelys.Desktop.csproj"

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $repositoryRoot "artifacts\poc\win-x64"
}

$stagingDirectory = Join-Path $OutputRoot "publish"
$deliveryDirectory = Join-Path $OutputRoot "delivery"
New-Item -ItemType Directory -Path $stagingDirectory -Force | Out-Null
New-Item -ItemType Directory -Path $deliveryDirectory -Force | Out-Null

$env:MSBuildEnableWorkloadResolver = "false"
$env:DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE = "true"

& dotnet publish $project `
    -c $Configuration `
    -r win-x64 `
    --self-contained true `
    -p:PublishSingleFile=true `
    -p:IncludeNativeLibrariesForSelfExtract=true `
    -p:EnableCompressionInSingleFile=true `
    -p:DebugType=embedded `
    -p:NuGetAudit=false `
    -o $stagingDirectory

if ($LASTEXITCODE -ne 0) {
    throw "Windows publish failed with exit code $LASTEXITCODE."
}

$publishedExecutable = Join-Path $stagingDirectory "Mnelys.exe"
$deliveryExecutable = Join-Path $deliveryDirectory "Mnelys.exe"
Copy-Item -LiteralPath $publishedExecutable -Destination $deliveryExecutable -Force

$unexpectedDeliveryFiles = Get-ChildItem -LiteralPath $deliveryDirectory -File |
    Where-Object Name -ne "Mnelys.exe"

if ($unexpectedDeliveryFiles) {
    throw "Delivery directory contains unexpected files: $($unexpectedDeliveryFiles.Name -join ', ')."
}

$file = Get-Item -LiteralPath $deliveryExecutable
$hash = Get-FileHash -LiteralPath $deliveryExecutable -Algorithm SHA256
Write-Output "Windows single-file artifact: $($file.FullName)"
Write-Output "Bytes: $($file.Length)"
Write-Output "SHA-256: $($hash.Hash)"
