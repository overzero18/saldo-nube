# Dependencias

## Dependencias incorporadas

### Infraestructura

- `rusqlite` 0.40.2, con `bundled-sqlcipher-vendored-openssl`: adaptador Rust para SQLite/SQLCipher. Compila SQLCipher y OpenSSL integrados para que Windows y Ubuntu no dependan de una instalación externa. Solo está declarado en `finanzas-infrastructure`.
  - Finalidad: base de datos local cifrada y migraciones transaccionales.
  - Red, telemetría y puertos: no aporta cliente de red ni servidor; la base de datos se abre como fichero local dentro del proceso.
  - Licencia: MIT para `rusqlite`; SQLite es de dominio público y SQLCipher usa licencia BSD de tres cláusulas. Se verificará el inventario final de licencias antes de distribuir.
  - Alternativa descartada: MariaDB, porque es un servidor independiente, requiere operación adicional y normalmente escucha en red; no encaja con el contrato local sin servicios.
  - Característica adicional `backup`: expone la API de copia en línea de SQLite para crear copias coherentes de una base que está abierta. No añade red ni procesos.

### Interfaz

## Dependencia incorporada y en revisión

- `slint` y `slint-build` 1.18.1: interfaz compilada para la aplicación de escritorio. Solo se declaran en `finanzas-desktop`; no se añaden al dominio, aplicación ni infraestructura.

Su grafo transitivo es amplio porque contiene adaptadores de escritorio y de compilación para Windows, Linux, macOS, Android y WebAssembly. La revisión local encontró `webbrowser` y canales de escritorio transitivos (por accesibilidad e integración de plataforma); la aplicación no los invoca ni contiene cliente HTTP, socket o código de apertura de URL. La URL de una comparación bancaria se guarda como texto y no es enlazable desde la interfaz. Esto reduce la superficie, pero no convierte el grafo transitivo en una garantía: se repetirá la inspección de listeners en cada binario final.

## Norma de incorporación

Antes de añadir una dependencia se documentarán su versión, licencia, finalidad, mantenimiento, superficie de red y alternativa descartada. Las dependencias que habiliten red, telemetría, procesos residentes o ejecución de código externo requieren aprobación expresa antes de incorporarse.
