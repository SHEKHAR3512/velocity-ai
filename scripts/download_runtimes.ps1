param(
    [switch]$IncludeCuda
)

$ErrorActionPreference = "Stop"
$tag = "b12220"
$binDir = Join-Path $PSScriptRoot "..\bin"
if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Path $binDir -Force | Out-Null
}

function Fetch-Runtime {
    param(
        [string]$FileName,
        [string]$ExtractSubDir
    )
    $url = "https://github.com/ggml-org/llama.cpp/releases/download/$tag/$FileName"
    $zipPath = Join-Path $binDir $FileName
    $targetDir = Join-Path $binDir $ExtractSubDir

    if (-not (Test-Path $targetDir)) {
        New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
    }

    if (-not (Test-Path (Join-Path $targetDir "llama-cli.exe")) -and -not (Test-Path (Join-Path $targetDir "llama-bench.exe"))) {
        Write-Host "==> Downloading $FileName via curl..."
        & curl.exe -L --progress-bar -f "$url" -o "$zipPath"
        if ($LASTEXITCODE -ne 0) {
            throw "curl download failed for $FileName (exit code $LASTEXITCODE)"
        }
        Write-Host "==> Extracting $FileName to $targetDir via tar..."
        & tar.exe -xf "$zipPath" -C "$targetDir"
        if ($LASTEXITCODE -ne 0) {
            # fallback to Expand-Archive if tar fails
            Expand-Archive -Path $zipPath -DestinationPath $targetDir -Force
        }
        Remove-Item -Path $zipPath -Force -ErrorAction SilentlyContinue
        Write-Host "✓ Extracted $FileName successfully."
    } else {
        Write-Host "✓ $ExtractSubDir already exists with binaries. Skipping download."
    }
}

Write-Host "=== Setting up llama.cpp CPU AVX2 Runtime ==="
Fetch-Runtime -FileName "llama-$tag-bin-win-cpu-x64.zip" -ExtractSubDir "llama-cpu"

Write-Host "`n=== Setting up llama.cpp Vulkan Runtime ==="
Fetch-Runtime -FileName "llama-$tag-bin-win-vulkan-x64.zip" -ExtractSubDir "llama-vulkan"

if ($IncludeCuda) {
    Write-Host "`n=== Setting up llama.cpp CUDA 12.4 Runtime ==="
    Fetch-Runtime -FileName "llama-$tag-bin-win-cuda-12.4-x64.zip" -ExtractSubDir "llama-cuda"

    $cudaTargetDir = Join-Path $binDir "llama-cuda"
    if (-not (Test-Path (Join-Path $cudaTargetDir "cudart64_12.dll"))) {
        Write-Host "`n=== Downloading CUDA 12.4 CUDART Dependencies ==="
        $cudartZip = "cudart-llama-bin-win-cuda-12.4-x64.zip"
        $cudartUrl = "https://github.com/ggml-org/llama.cpp/releases/download/$tag/$cudartZip"
        $cudartPath = Join-Path $binDir $cudartZip
        & curl.exe -L --progress-bar -f "$cudartUrl" -o "$cudartPath"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "==> Extracting $cudartZip to $cudaTargetDir..."
            & tar.exe -xf "$cudartPath" -C "$cudaTargetDir"
            Remove-Item -Path $cudartPath -Force -ErrorAction SilentlyContinue
            Write-Host "✓ CUDA 12.4 runtime ready."
        }
    }
}

Write-Host "`n=== All requested runtimes successfully configured! ==="
