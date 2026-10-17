# Persistencia local

## Objetivo

Guardar movimientos, suscripciones, cuentas y preferencias en una única base de datos local cifrada. No habrá sincronización, cuenta de usuario, red ni copia en la nube.

## Reglas

- La clave de cifrado se solicita localmente y no se guarda en texto plano, registros, capturas ni Git.
- La base de datos se abre solo mientras la aplicación está desbloqueada.
- Los datos de ejemplo y pruebas nunca reutilizan ficheros de usuario.
- Las migraciones serán numeradas, transaccionales y probadas antes de aplicarse a datos reales.
- La copia de seguridad es una acción explícita de la persona usuaria, también cifrada. Se realiza con la API de copia en línea de SQLite, no copiando el archivo abierto.

## Ubicación de datos

- Windows: `%LOCALAPPDATA%/saldo-nube/finanzas.db`.
- Ubuntu y otros escritorios Linux: `$XDG_DATA_HOME/saldo-nube/finanzas.db`, o `~/.local/share/saldo-nube/finanzas.db` si no existe la primera variable.

Al actualizar desde el prototipo local anterior, la aplicación detecta primero la base existente en la carpeta histórica y la sigue abriendo, para no ocultar datos ya creados. Las instalaciones nuevas usan `saldo-nube`.

La carpeta se crea únicamente después de validar una contraseña no vacía y, en una base nueva, la confirmación de esa contraseña.

## Esquema inicial aplicado

- `schema_migrations`: versión de cada migración aplicada.
- `accounts`: etiqueta de cuenta manual, sin IBAN, credenciales ni conexión bancaria.
- `movements`: importe en céntimos, tipo, categoría, descripción y fecha local.
- `subscriptions`: importe, frecuencia y próximo cobro.
- `preferences`: únicamente preferencias visuales no sensibles.
- `category_budgets`: categoría, límite en céntimos y mes local de aplicación.
- `savings_goals`: nombre, cantidad objetivo y cantidad ahorrada, ambas en céntimos.
- `portfolio_holdings`: posiciones de cartera anotadas manualmente, con importe invertido, valor manual y fecha de revisión.
- `bank_offers`: comparativas transparentes introducidas por la persona usuaria, con comisión, condiciones, URL oficial HTTPS y fecha de consulta.

Las migraciones 2 y 3 crean estas tablas de forma transaccional y quedan registradas. Las bases existentes reciben las migraciones al desbloquearse, sin modificar sus movimientos ni suscripciones.

## Verificación realizada

1. Se creó una base SQLCipher temporal cifrada desde la prueba automática.
2. Se añadió un movimiento ficticio y se leyó tras cerrar y reabrir el fichero.
3. Se confirmó que una clave diferente no permite leer el contenido.
4. La migración 1 se aplica dentro de una transacción y se registra una única vez.
5. Una prueba crea una copia SQLCipher, la abre con la contraseña y comprueba que contiene el movimiento original.
6. Una prueba simula una base existente en migración 2, aplica la migración 3 y confirma que conserva su movimiento anterior.

## Copias cifradas

La acción «Copia cifrada» pide de nuevo la contraseña, la borra de la interfaz tras usarla y crea un fichero nuevo en el directorio `backups` junto a la base. El nombre incorpora un identificador temporal para que no se sobrescriba una copia previa.

## Límites antes de introducir datos reales

1. La cartera no descarga cotizaciones ni ejecuta operaciones: los valores son siempre manuales.
2. La comparativa bancaria no puntúa ni recomienda entidades; exige URL HTTPS oficial y fecha de consulta visibles.
3. Repite la comprobación de listeners y tráfico de red en el binario final de cada plataforma.
