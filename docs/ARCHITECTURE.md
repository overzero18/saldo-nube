# Arquitectura

La aplicación se organizará en cuatro capas: dominio, aplicación, infraestructura y UI. La UI Slint nunca contendrá reglas de cálculo ni consultas SQL. La lógica de dominio se probará sin interfaz gráfica ni base de datos.

Todo cambio que cruce capas debe describir su contrato y contar con pruebas.

## Estado del esqueleto

El workspace Rust ya contiene `domain`, `application`, `infrastructure` y `desktop`. El dominio contiene importes en céntimos, fechas locales, movimientos, suscripciones y cálculos; la aplicación define el puerto `LedgerRepository`; infraestructura ofrece tanto una implementación temporal en memoria como `SqlCipherLedger`, el adaptador local cifrado. El puerto incluye alta, listado y borrado explícito de movimientos. La dependencia SQLCipher queda aislada en infraestructura: el dominio, la aplicación y la UI no importan SQL ni claves.

El dominio también contiene `CategoryBudget`, su progreso mensual reproducible a partir de movimientos, `SavingsGoal`, `PortfolioHolding` y `BankOffer`. La infraestructura persiste presupuestos y objetivos mediante la migración 2, y cartera/comparativas mediante la migración 3. La comparativa exige una fuente HTTPS oficial y fecha visibles; no llama a esa URL. El contrato añade además actualización dirigida de un movimiento: solo modifica el registro con el identificador concreto indicado.
