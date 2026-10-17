# Seguridad

## Comportamiento de red

La primera versión no realiza tráfico de red, no abre puertos y no instala procesos o servicios residentes. No se aceptará una dependencia que use red sin revisión explícita del diseño y de este documento.

La ventana inicial Slint se compiló y se ejecutó en Ubuntu el 2026-10-17. Mientras estuvo abierta se revisaron los listeners del sistema: no apareció ningún listener asociado a `finanzas-desktop`. La compilación de distribución y las pruebas se repitieron el mismo día; la comprobación `ss` disponible en el entorno aislado no pudo asociar procesos por restricciones de netlink, por lo que la verificación definitiva debe repetirse en el equipo destino, antes y durante la ejecución.

## Datos

No se solicitan ni almacenan credenciales bancarias. Las cuentas se crean manualmente y los datos permanecen locales. No incluyas datos reales, claves ni ficheros de base de datos en incidencias, ejemplos o commits.

La cartera acepta únicamente valores introducidos por la persona usuaria; no descarga cotizaciones ni permite comprar, vender o conectar un bróker. La comparación de cuentas solo conserva campos manuales, una URL HTTPS oficial y una fecha de consulta; no clasifica bancos ni transmite esos datos a la URL.

La persistencia usa SQLCipher embebido. La clave se entrega en un diálogo local, se limpia de la interfaz tras usarla y no se guarda en tablas, preferencias, registros ni código. Una clave diferente es rechazada antes de leer el esquema. La aplicación crea el fichero solo tras validar una clave y su confirmación inicial.

La lista de movimientos muestra un número explícito para cada registro. Editar requiere indicar ese número; eliminarlo exige una segunda confirmación que vuelve a mostrarlo. Las copias de seguridad cifradas se implementaron antes de habilitar estas correcciones, y las operaciones usan el identificador local cifrado del registro seleccionado, no texto libre ni una consulta global.

El repositorio ignora de forma preventiva las bases de datos, el directorio de datos y ficheros habituales de secretos.

## Verificación en Linux

Antes y después de ejecutar una versión, puede verificarse que no aparezcan listeners nuevos con:

```bash
ss -lntup
```

En sistemas donde `ss` no pueda mostrar PID o proceso por permisos, ejecútalo desde una terminal con los permisos habituales del usuario administrador y compara la salida antes y durante la aplicación.

## Avisos de seguridad

Mientras el repositorio no esté publicado, informa de problemas por el canal privado acordado con el mantenedor. La política de divulgación responsable está en [`SECURITY.md`](../SECURITY.md) y la licencia GPL-3.0-or-later está incluida como `LICENSE`.
