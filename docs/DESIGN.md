# Dirección visual

## Propuesta inicial

Una aplicación de escritorio serena, clara y densa en información útil, sin imitar una web bancaria ni añadir gráficos decorativos.

## Identidad

La identidad final es **Saldo Nube**. Su marca utiliza un icono propio de monedas apiladas y línea de progreso, en grafito, azul y verde; no contiene personajes, mascotas ni elementos de marcas de terceros. El icono se incluye como `assets/saldo-nube-mark.png` y el nombre se renderiza como texto nativo para mantener legibilidad.

```text
┌──────────────┬──────────────────────────────────────────────────────┐
│ Resumen      │ Octubre 2026                         + Movimiento    │
│ Movimientos  │ ┌─────────┐ ┌─────────┐ ┌──────────────────────────┐ │
│ Suscripciones│ │ Ingresos│ │ Gastos  │ │ Disponible este mes      │ │
│ Presupuestos │ └─────────┘ └─────────┘ └──────────────────────────┘ │
│ Cuentas      │                                                      │
│ Cartera      │ Movimientos recientes       Suscripciones próximas   │
│ Ajustes      │ Categorías y progreso de presupuesto                 │
└──────────────┴──────────────────────────────────────────────────────┘
```

- Barra lateral estrecha con texto e iconos; nunca muestra saldo sensible si la ventana no está desbloqueada.
- Cabecera con periodo actual y un único botón principal: `+ Movimiento`.
- Tres tarjetas: ingresos, gastos y disponible. Los porcentajes solo se mostrarán cuando exista comparación real con un periodo anterior.
- Movimientos y suscripciones tienen prioridad sobre gráficos. Los gráficos se añaden cuando haya datos suficientes.
- Paleta propuesta: grafito, superficies gris azulado, texto cálido, verde para ingresos, rojo suave para gastos y ámbar para avisos. El modo claro debe tener el mismo contraste y jerarquía.
- Temas confirmados: oscuro elegante por defecto y claro inspirado en Windows 11. Un selector permite alternar entre ambos sin modificar datos ni cálculos.
- Los botones son componentes propios: el principal usa azul de acción con texto blanco y el secundario usa la superficie del tema, borde visible y texto contrastado. No dependen del estilo de controles del sistema.
- La cabecera separa sus acciones 12 píxeles y reserva anchura adicional al selector de tema para evitar controles visualmente pegados.

## Referencias estudiadas

- [Budget Analytics Dashboard, Dimitri Honcharenko](https://dribbble.com/shots/21470736-Income-Analytics-Tool-Dashboard): buena jerarquía de resumen, movimientos y pagos próximos; se simplificará para evitar sobrecarga.
- [Personal Finance Dashboard](https://personalfinancedasboard.com/): navegación de escritorio clara y métricas separadas.
- [Onyx Workspace Expense Tracker](https://onyxworkspace.app/index.html): buen equilibrio entre tarjetas, categorías y lista de movimientos.

Las referencias son inspiración de estructura visual; no se reutilizarán sus iconos, imágenes, textos ni código.
## Campos de entrada

Los campos son componentes propios y no dependen del tema nativo. En modo claro usan fondo azul grisáceo suave, borde visible y texto oscuro; en modo oscuro, fondo profundo y texto claro. Así se mantiene contraste suficiente para contraseña, categoría, descripción, importe y fecha.

