# scripts/setup_tools.ps1
# Script automatizado para baixar e configurar as dependências portáteis do Mirro (scrcpy + adb)

param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$ProjectRoot = (Get-Item "$PSScriptRoot\..").FullName
$ToolsDir = Join-Path $ProjectRoot "tools"
$ScrcpyTargetDir = Join-Path $ToolsDir "scrcpy"
$ScrcpyExe = Join-Path $ScrcpyTargetDir "scrcpy.exe"
$AdbExe = Join-Path $ScrcpyTargetDir "adb.exe"

if (-not $Force -and (Test-Path $ScrcpyExe) -and (Test-Path $AdbExe)) {
    Write-Host "[Mirro] As ferramentas já estão instaladas em '$ScrcpyTargetDir'." -ForegroundColor Green
    exit 0
}

Write-Host "[Mirro] Configurando dependências portáteis (scrcpy + adb)..." -ForegroundColor Cyan

if (-not (Test-Path $ToolsDir)) {
    New-Item -ItemType Directory -Path $ToolsDir | Out-Null
}

$ZipPath = Join-Path $ToolsDir "scrcpy-win64.zip"
$ExtractTemp = Join-Path $ToolsDir "scrcpy_temp"

try {
    Write-Host "[Mirro] Consultando versão mais recente do scrcpy no GitHub..." -ForegroundColor Gray
    $downloadUrl = "https://github.com/Genymobile/scrcpy/releases/download/v5.0.1/scrcpy-win64-v5.0.1.zip"
    
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/Genymobile/scrcpy/releases/latest" -Headers @{"User-Agent"="Mirro-Setup"}
        $asset = $release.assets | Where-Object { $_.name -like "scrcpy-win64-*.zip" } | Select-Object -First 1
        if ($asset) {
            $downloadUrl = $asset.browser_download_url
            Write-Host "[Mirro] Versão encontrada: $($release.tag_name) ($($asset.name))" -ForegroundColor Gray
        }
    } catch {
        Write-Warning "[Mirro] Não foi possível consultar a API do GitHub. Usando URL direta de fallback ($downloadUrl)."
    }

    Write-Host "[Mirro] Baixando pacote portátil de $downloadUrl..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $downloadUrl -OutFile $ZipPath -UseBasicParsing

    Write-Host "[Mirro] Extraindo arquivos..." -ForegroundColor Cyan
    if (Test-Path $ExtractTemp) {
        Remove-Item -Recurse -Force $ExtractTemp
    }
    Expand-Archive -Path $ZipPath -DestinationPath $ExtractTemp -Force

    # Encontra o diretório interno extraído (ex: scrcpy-win64-v5.0.1)
    $innerDir = Get-ChildItem -Path $ExtractTemp -Directory | Select-Object -First 1
    if (-not $innerDir) {
        throw "Diretório do scrcpy não encontrado dentro do arquivo ZIP."
    }

    if (Test-Path $ScrcpyTargetDir) {
        Remove-Item -Recurse -Force $ScrcpyTargetDir
    }

    Move-Item -Path $innerDir.FullName -Destination $ScrcpyTargetDir

    Write-Host "[Mirro] Limpando arquivos temporários..." -ForegroundColor Gray
    Remove-Item -Force $ZipPath
    Remove-Item -Recurse -Force $ExtractTemp

    Write-Host "[Mirro] Sucesso! scrcpy e adb instalados em '$ScrcpyTargetDir'." -ForegroundColor Green
} catch {
    Write-Error "[Mirro] Falha ao configurar dependências: $_"
    exit 1
}
