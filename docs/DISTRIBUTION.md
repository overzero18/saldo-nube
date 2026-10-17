# Empaquetado local

## Principio

El binario de distribución sigue siendo una aplicación local: no instala servicios, no abre puertos y no configura inicio automático. La base cifrada se crea únicamente al desbloquearla desde la propia interfaz.

## Ubuntu

En una máquina de compilación con Rust y las dependencias de Slint disponibles:

```bash
cargo build --release -p finanzas-desktop
```

El ejecutable queda en `target/release/finanzas-desktop`. Antes de entregarlo se debe comprobar:

```bash
ss -lntup
```

antes y durante su ejecución, confirmando que no aparece ningún listener asociado.

La verificación local de 2026-10-17 compiló `target/release/finanzas-desktop` (aprox. 32 MiB), ejecutó las pruebas del workspace y comprobó un proceso `finanzas-desktop` abierto sin listener asociado en `ss -lntup`. El hash SHA-256 depende de cada compilación y se debe calcular de nuevo antes de distribuir un binario.

## Windows 11

La compilación de distribución se realizará desde Windows 11 o desde una cadena de compilación Windows verificada:

```powershell
cargo build --release -p finanzas-desktop
```

No se añadirá instalador, tarea programada ni servicio hasta revisar expresamente su contenido. La primera entrega podrá ser un archivo comprimido con el ejecutable y esta documentación.

Esta máquina de verificación usa el objetivo `x86_64-unknown-linux-gnu` y no dispone de objetivo o herramienta de compilación Windows instalados; por tanto no sustituye la prueba real de Windows 11.

Para hacer la comprobación reproducible en Windows, desde PowerShell y la raíz del proyecto:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
.\scripts\windows-verify.ps1
```

El script ejecuta pruebas, genera el binario, abre la aplicación y comprueba con `Get-NetTCPConnection` que el PID iniciado no tiene listeners TCP. No instala servicios ni detiene procesos. Después hay que desbloquear una base de prueba, revisar la interfaz y cerrar la ventana manualmente.

## Antes de publicar

1. Repetir pruebas completas y comprobación de listeners en Ubuntu y Windows.
2. Generar una copia cifrada de prueba y verificar restauración local.
3. Incluir licencia completa elegida por el mantenedor y política `SECURITY.md` de divulgación.
4. Revisar dependencias y confirmar que no incorporan telemetría, conexión bancaria ni red.
5. Alcanzar el umbral acordado del 90 % antes de crear o enlazar GitHub.
