# Ejecutar desde la raíz del proyecto en Windows 11 con Rust/Cargo instalados.
# No instala servicios, no modifica la configuración de red y deja la ventana abierta para revisión manual.
$ErrorActionPreference = "Stop"

cargo test --workspace
cargo build --release -p finanzas-desktop

$binary = Join-Path $PSScriptRoot "..\target\release\finanzas-desktop.exe"
if (-not (Test-Path $binary)) {
    throw "No se encontró el binario de distribución: $binary"
}

$before = @(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue)
$process = Start-Process -FilePath $binary -PassThru
Start-Sleep -Seconds 2
$during = @(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue)
$ownedListeners = $during | Where-Object { $_.OwningProcess -eq $process.Id }

if ($ownedListeners) {
    $ownedListeners | Format-Table -AutoSize | Out-String | Write-Error
    throw "El proceso de Finanzas abrió un listener inesperado. Ciérralo y revisa el resultado antes de distribuir."
}

Write-Host "Pruebas y compilación completadas. El proceso $($process.Id) no tiene listeners TCP asociados."
Write-Host "La aplicación sigue abierta para revisar la interfaz; ciérrala manualmente al terminar."
