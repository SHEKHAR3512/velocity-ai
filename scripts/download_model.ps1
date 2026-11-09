param(
    [string]$ModelName = "qwen2.5-0.5b-instruct-q4_k_m.gguf",
    [string]$Url = "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf"
)

$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$modelsDir = Join-Path $PSScriptRoot "..\models"
if (-not (Test-Path $modelsDir)) {
    New-Item -ItemType Directory -Path $modelsDir -Force | Out-Null
}

$destPath = Join-Path $modelsDir $ModelName
$partPath = "$destPath.part"

if (Test-Path $destPath) {
    $existingSize = (Get-Item $destPath).Length
    Write-Host "Model $ModelName already exists ($([math]::Round($existingSize/1MB, 2)) MB). Skipping download."
    exit 0
}

Write-Host "Downloading $ModelName from $Url..."
Write-Host "Target location: $destPath"

# Use .NET HttpClient for reliable streaming download
$client = [System.Net.Http.HttpClient]::new()
$client.Timeout = [TimeSpan]::FromMinutes(30)

$response = $client.GetAsync($Url, [System.Net.Http.HttpCompletionOption]::ResponseHeadersRead).GetAwaiter().GetResult()
if (-not $response.IsSuccessStatusCode) {
    throw "Failed to download model. HTTP status: $($response.StatusCode)"
}

$totalBytes = $response.Content.Headers.ContentLength
$totalMB = [math]::Round($totalBytes / 1MB, 2)
Write-Host "Total file size: $totalMB MB"

$inStream = $response.Content.ReadAsStreamAsync().GetAwaiter().GetResult()
$outStream = [System.IO.File]::Create($partPath)
$buffer = [byte[]]::new(1048576) # 1 MB buffer
$downloaded = 0
$lastLogTime = [DateTime]::UtcNow

while (($read = $inStream.Read($buffer, 0, $buffer.Length)) -gt 0) {
    $outStream.Write($buffer, 0, $read)
    $downloaded += $read
    $now = [DateTime]::UtcNow
    if (($now - $lastLogTime).TotalSeconds -ge 3) {
        $lastLogTime = $now
        $pct = if ($totalBytes -gt 0) { [math]::Round(($downloaded / $totalBytes) * 100, 1) } else { 0 }
        $currMB = [math]::Round($downloaded / 1MB, 2)
        Write-Host "Downloaded: $currMB MB / $totalMB MB ($pct%)..."
    }
}

$outStream.Dispose()
$inStream.Dispose()
$client.Dispose()

Move-Item -Path $partPath -Destination $destPath -Force
$finalSize = (Get-Item $destPath).Length
Write-Host "Model download complete! Verified file size: $([math]::Round($finalSize/1MB, 2)) MB."

