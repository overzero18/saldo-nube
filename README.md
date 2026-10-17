# Saldo Nube

Aplicación de escritorio local para registrar gastos, ingresos, suscripciones y objetivos de ahorro en Windows 11 y Ubuntu.

Aplicación con identidad propia: el nombre y el icono de monedas y progreso son originales del proyecto.

## Estado

Versión inicial pública: pide desbloqueo local, cifra la base de datos con SQLCipher y permite guardar movimientos, suscripciones, presupuestos, objetivos, cartera y comparativas manuales, además de crear copias cifradas. No solicita permisos especiales, salvo acceso normal al directorio privado de datos del usuario al crear una base.

El código se publicó después de superar el umbral local acordado, la revisión de seguridad y dieciocho pruebas en Ubuntu. Sigue pendiente ejecutar la validación de distribución en un Windows 11 real mediante [`scripts/windows-verify.ps1`](scripts/windows-verify.ps1).

## Principios

- Sin conexión a bancos, nube, telemetría ni red.
- Sin puertos abiertos ni servicios en segundo plano.
- Lógica financiera en Rust independiente de la interfaz Slint.
- Datos introducidos manualmente y almacenados de forma local.
- No proporciona asesoramiento personalizado de inversión.

Consulta [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/SECURITY.md](docs/SECURITY.md), [SECURITY.md](SECURITY.md), [docs/PERSISTENCE.md](docs/PERSISTENCE.md), [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md), [docs/DESIGN.md](docs/DESIGN.md), [docs/DISTRIBUTION.md](docs/DISTRIBUTION.md) y [docs/CHANGELOG.md](docs/CHANGELOG.md).

## Licencia

El código se distribuye bajo [GPL-3.0-or-later](LICENSE). La licencia completa ya forma parte del árbol local antes de una futura publicación.
