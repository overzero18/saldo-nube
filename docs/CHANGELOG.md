# Registro de cambios

Este archivo se actualiza en cada cambio funcional, técnico o de seguridad.

## Sin publicar

### Añadido — 2026-10-17

- Estructura documental inicial.
- Contrato de ejecución local sin red, puertos ni servicios residentes.
- Arquitectura prevista con lógica Rust separada de la interfaz Slint.
- Protección inicial de Git para impedir versionar bases de datos y secretos locales.
- Workspace Rust con capas `domain`, `application`, `infrastructure` y `desktop`.
- Tipo monetario exacto en céntimos y pruebas unitarias iniciales.
- Cadena de compilación Rust 1.93 y formateador `rustfmt` verificados en Ubuntu.
- Regla de publicación: no se creará ni enlazará un repositorio de GitHub antes de completar al menos el 90 % del proyecto y revisar seguridad y documentación.
- Movimientos, cuentas etiquetadas, suscripciones y fechas locales validados en el dominio, sin banco ni red.
- Puerto de persistencia separado y adaptador temporal en memoria para pruebas sin base de datos.
- Prueba de integración entre caso de uso y repositorio local temporal.
- Inventario y norma de revisión de dependencias antes de incorporarlas.
- Declaradas y descargadas las dependencias de interfaz `slint` y `slint-build` 1.18.1, pendientes de validación de compilación y revisión de su grafo transitivo.
- Dirección visual inicial documentada con una identidad propia inspirada en patrones públicos de paneles de finanzas.
- Base visual propia implementada con dos temas: oscuro elegante y claro de alto contraste.
- Botones de acción propios para mantener contraste y jerarquía coherentes, especialmente en modo claro.
- Corregido un conflicto de nombre interno de Slint en el componente de botones antes de compilar.
- Aumentado el espaciado y la anchura de los botones de cabecera.
- Formulario local para añadir gastos e ingresos manuales, con validación de importe y fecha y resumen actualizado en memoria.
- Contrato de persistencia cifrada, migraciones y copias de seguridad documentado antes de incorporar una biblioteca externa.
- Interfaz Slint compilada y arrancada correctamente en Ubuntu tras instalar las herramientas de desarrollo de Fontconfig.
- Verificación durante la ejecución: no apareció ningún puerto en escucha asociado a la aplicación.
- Seleccionado SQLCipher integrado mediante `rusqlite` para la futura persistencia cifrada, local y multiplataforma; no se incorpora ningún servidor de base de datos.
- Implementado `SqlCipherLedger`: crea o abre una base SQLCipher local, aplica la migración transaccional inicial y persiste movimientos y suscripciones mediante el contrato de aplicación.
- Añadidas pruebas de persistencia cifrada: reapertura correcta y rechazo de una clave distinta. Total actual: ocho pruebas automatizadas superadas.
- Añadido desbloqueo local en la interfaz y conexión del formulario de movimientos a `SqlCipherLedger`; los datos sobreviven al reinicio sin conservar la clave.
- Definida la ubicación privada de la base en Windows y Linux; no se crea ninguna carpeta de datos hasta validar la contraseña inicial.
- Aplicado temporalmente el nombre local «Finanzas con Snoopy». Antes de cualquier publicación deberá sustituirse por una marca propia o verificarse autorización de uso.
- Corregida la visibilidad de todos los campos en modo claro mediante componentes de entrada propios con contraste explícito.
- El panel ya muestra fecha, tipo, categoría, descripción e importe de todos los movimientos guardados al desbloquear la base.
- Añadida eliminación confirmada del último movimiento. El contrato de aplicación y los adaptadores en memoria y SQLCipher eliminan exclusivamente el identificador solicitado; prueba automatizada incluida. Total actual: nueve pruebas superadas.
- Añadidas copias de seguridad SQLCipher explícitas y coherentes mediante la API de backup de SQLite. Requieren introducir la contraseña, no sobrescriben ficheros y se comprueban abriendo una copia en las pruebas. Total actual: diez pruebas superadas.
- Añadido formulario local para suscripciones mensuales y anuales con próximo cobro, y listado de próximos pagos. La persistencia cifrada de suscripciones cuenta con prueba automática. Total actual: once pruebas superadas.
- Añadidas reglas puras para presupuestos por categoría y mes, y objetivos de ahorro. Incluyen pruebas para el filtrado mensual y el cálculo de objetivo completado. Total actual: trece pruebas superadas.
- Añadida migración SQLCipher 2 y adaptadores de repositorio para presupuestos y objetivos de ahorro. Las tablas se crean de forma transaccional y no alteran datos ya existentes.
- Añadida pantalla local «Presupuestos y ahorro»: registra presupuestos de categoría para octubre de 2026 y objetivos de ahorro, y muestra su progreso cifrado.
- El formulario de movimientos acepta una cuenta manual. La etiqueta se almacena localmente y se reutiliza; no se solicitan IBAN ni credenciales.
- Documentado el empaquetado local para Ubuntu y Windows 11, con comprobación de listeners y sin instaladores, servicios o publicación remota.
- Añadido resumen local de gastos por categoría, calculado solo desde los movimientos cifrados del dispositivo.
- Añadido contrato de actualización dirigida de movimientos y adaptadores SQLCipher/en memoria; la interfaz de selección se completará antes de exponerlo.
- El periodo de los presupuestos ya se elige con mes y año, en vez de quedar fijado a una fecha concreta.
- Añadida preparación para editar el último movimiento guardado desde la interfaz; la actualización mantiene el mismo identificador cifrado.
- Ajustados los espacios en los paneles «Movimientos guardados» y «Próximos pagos» para que los botones no queden pegados.
- Añadido el texto completo de licencia GPL-3.0-or-later al árbol local.
- Validada la compilación de distribución en Ubuntu: `cargo build --release -p finanzas-desktop` creó un binario local de aproximadamente 32 MiB y `cargo test --workspace` supera ahora catorce pruebas.
- La lista de movimientos ahora numera los registros y permite editar o eliminar el que se indique; el borrado conserva una confirmación que muestra el número seleccionado.
- Reconstruida y abierta la distribución actual de Ubuntu para la comprobación visual; su proceso no apareció asociado a ningún listener en `ss -lntup`.
- Añadida una prueba de interfaz para la selección uno-basada de un movimiento y el rechazo de números inválidos.
- Añadida una política local de divulgación responsable que prohíbe incluir claves, bases o datos personales en informes de seguridad.
- Añadidas cartera manual y comparación transparente de cuentas: sus datos se cifran en SQLCipher mediante migración 3, no realizan cotizaciones ni conexiones bancarias, y la comparación exige fuente HTTPS oficial y fecha de consulta.
- La suite suma dieciocho pruebas: incluye la reapertura SQLCipher de cartera y comparativas, y verifica que la migración 3 conserva movimientos existentes.
- Auditoría local final: no hay remoto Git configurado, el binario Linux release fue ejecutado sin listener asociado y el objetivo Windows no está instalado en este entorno, por lo que su validación sigue siendo una tarea de plataforma separada.
- Añadido `scripts/windows-verify.ps1` para que la validación pendiente de Windows 11 ejecute las mismas pruebas, genere la release y compruebe listeners del PID de la aplicación sin instalar servicios.
- La identidad pública pasa a ser «Saldo Nube», con icono original de monedas y progreso. La ruta de datos mantiene compatibilidad de lectura con el prototipo local anterior.
- Publicado el repositorio inicial de código revisado en GitHub, sin datos locales, copias cifradas, credenciales ni binarios.
