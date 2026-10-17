<p align="center">
  <img src="assets/saldo-nube-mark.png" width="150" alt="Icono de Saldo Nube: monedas y línea de progreso">
</p>

<h1 align="center">Saldo Nube</h1>

<p align="center">
  <strong>Tus finanzas, en tu equipo y bajo tu control.</strong><br>
  Escritorio local · base cifrada · sin conexión bancaria
</p>

---

Llevar tus gastos no debería obligarte a entregar tu información financiera a una nube. **Saldo Nube** es una aplicación de escritorio para anotar, entender y planificar tus finanzas manualmente: los datos se cifran en una base local y la aplicación no abre puertos, no muestra anuncios ni incorpora telemetría.

## Por qué Saldo Nube

- **Tus datos se quedan contigo.** Sin cuentas de usuario, sincronización, nube ni conexión a bancos.
- **Cifrado local.** La base usa SQLCipher; la contraseña se solicita en el dispositivo y no se guarda.
- **Información útil, sin ruido.** Registra ingresos, gastos, suscripciones, presupuestos y objetivos de ahorro.
- **Cartera sin consejos disfrazados.** Anota posiciones y valores manualmente, sin cotizaciones automáticas ni recomendaciones de compra o venta.
- **Comparativas transparentes.** Guarda comisiones, condiciones, fuente oficial y fecha de consulta de cada cuenta que compares.

## Qué puedes hacer

| Área | Incluye |
| --- | --- |
| Movimientos | Gastos e ingresos, categorías, fecha, descripción y cuenta manual. |
| Suscripciones | Cobros mensuales o anuales y próximos pagos. |
| Planificación | Presupuestos mensuales por categoría y objetivos de ahorro. |
| Cartera | Posiciones, importe invertido, valor anotado y resultado manual. |
| Cuentas | Comparativas con comisión, condiciones, URL HTTPS oficial y fecha de revisión. |
| Protección | Copias SQLCipher explícitas que no sobrescriben archivos anteriores. |

## Lo que deliberadamente no hace

- No pide credenciales bancarias, IBAN ni acceso a tu banco.
- No envía tus movimientos, contraseñas ni comparativas a ningún servidor.
- No abre puertos ni instala servicios en segundo plano.
- No ejecuta operaciones ni ofrece asesoramiento financiero personalizado.

## Empieza en Ubuntu

Necesitas Rust y las dependencias de escritorio que requiere Slint. Desde la raíz del proyecto:

```bash
cargo run -p finanzas-desktop
```

La primera vez, crea una contraseña para la base local. Después añade un movimiento manual y utiliza **Copia cifrada** antes de introducir información que quieras conservar.

Para compilar una versión optimizada:

```bash
cargo build --release -p finanzas-desktop
```

## Estado y plataformas

La versión inicial pública ha superado **18 pruebas** en Ubuntu, incluidas persistencia SQLCipher, copias cifradas y migraciones de datos existentes. La aplicación se ha ejecutado localmente sin listeners asociados.

- **Ubuntu:** compilada y verificada localmente.
- **Windows 11:** el código y el script de validación están preparados; falta la ejecución final en un Windows real antes de publicar una release distribuible. Consulta [`scripts/windows-verify.ps1`](scripts/windows-verify.ps1).

## Arquitectura que se puede revisar

La lógica financiera está separada de la interfaz:

```text
domain  → reglas financieras puras
application → casos de uso y contratos
infrastructure → SQLCipher, migraciones y copias locales
desktop → interfaz Slint
```

No hay dependencia de la interfaz en las reglas de dinero, fechas o presupuestos. Los importes se trabajan en céntimos para evitar errores de coma flotante.

## Seguridad y documentación

- [Arquitectura](docs/ARCHITECTURE.md)
- [Persistencia y copias cifradas](docs/PERSISTENCE.md)
- [Seguridad](docs/SECURITY.md)
- [Política de divulgación](SECURITY.md)
- [Dependencias](docs/DEPENDENCIES.md)
- [Empaquetado](docs/DISTRIBUTION.md)
- [Registro de cambios](docs/CHANGELOG.md)

Si detectas un problema de seguridad, no publiques contraseñas, bases SQLCipher ni datos reales. Revisa primero la [política de seguridad](SECURITY.md).

## Licencia

Saldo Nube se distribuye bajo [GPL-3.0-or-later](LICENSE).
