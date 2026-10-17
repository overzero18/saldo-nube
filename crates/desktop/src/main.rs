//! Punto de entrada de escritorio. No abre red ni persiste datos sin desbloqueo explícito.
use finanzas_application::{
    LedgerRepository, calculate_current_summary, record_bank_offer, record_category_budget,
    record_movement, record_portfolio_holding, record_savings_goal, record_subscription,
    remove_movement, revise_movement,
};
use finanzas_domain::{
    BankOffer, BillingFrequency, CategoryBudget, LocalDate, Money, Movement, MovementKind,
    PortfolioHolding, SavingsGoal, Subscription,
};
use finanzas_infrastructure::{SqlCipherLedger, SqlCipherLedgerError};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs,
    path::PathBuf,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;
    let weak = window.as_weak();
    window.on_toggle_theme(move || {
        if let Some(w) = weak.upgrade() {
            w.set_dark_mode(!w.get_dark_mode())
        }
    });
    let ledger = Rc::new(RefCell::new(None::<SqlCipherLedger>));
    let backup_ledger = Rc::clone(&ledger);
    let weak = window.as_weak();
    window.on_create_backup(move |passphrase| {
        let Some(w) = weak.upgrade() else { return };
        if passphrase.is_empty() {
            w.set_backup_status("Introduce la contraseña para cifrar la copia".into());
            return;
        }
        let path = backup_path();
        let Some(directory) = path.parent() else {
            w.set_backup_status("No se pudo preparar la carpeta de copias".into());
            return;
        };
        if fs::create_dir_all(directory).is_err() {
            w.set_backup_status("No se pudo preparar la carpeta de copias".into());
            return;
        }
        let active = backup_ledger.borrow();
        let Some(active) = active.as_ref() else {
            w.set_backup_status("Desbloquea primero tu espacio local".into());
            return;
        };
        match active.create_encrypted_backup(&path, &passphrase) {
            Ok(()) => {
                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("copia.db");
                w.set_backup_password_input("".into());
                w.set_backup_status(format!("Copia creada: {filename}").into());
            }
            Err(SqlCipherLedgerError::EmptyKey) => {
                w.set_backup_status("Introduce la contraseña para cifrar la copia".into())
            }
            Err(SqlCipherLedgerError::BackupAlreadyExists) => {
                w.set_backup_status("La copia ya existe; vuelve a intentarlo".into())
            }
            Err(_) => w.set_backup_status("No se pudo crear la copia cifrada".into()),
        }
    });
    let delete_ledger = Rc::clone(&ledger);
    let edit_ledger = Rc::clone(&ledger);
    let weak = window.as_weak();
    window.on_edit_movement(move |number| {
        let Some(w) = weak.upgrade() else { return };
        let active = edit_ledger.borrow();
        let Some(active) = active.as_ref() else {
            return;
        };
        let Ok(position) = parse_movement_position(&number) else {
            w.set_delete_status("Indica un número de movimiento válido".into());
            return;
        };
        if let Ok(Some(movement)) = active
            .list_movements()
            .map(|items| movement_at_position(items, position))
        {
            w.set_editing_movement_id(movement.id.into());
            w.set_account_input(movement.account_id.into());
            w.set_category_input(movement.category.into());
            w.set_description_input(movement.description.into());
            w.set_amount_input(money_input(movement.amount).into());
            w.set_date_input(movement.occurred_on.to_iso8601().into());
            w.set_form_open(true);
        } else {
            w.set_delete_status("No existe un movimiento con ese número".into());
        }
    });
    let weak = window.as_weak();
    window.on_delete_movement(move |number| {
        let Some(w) = weak.upgrade() else { return };
        let mut active = delete_ledger.borrow_mut();
        let Some(active) = active.as_mut() else {
            w.set_delete_status("Desbloquea primero tu espacio local".into());
            return;
        };
        let position = match parse_movement_position(&number) {
            Ok(position) => position,
            Err(_) => {
                w.set_delete_status("Indica un número de movimiento válido".into());
                w.set_delete_confirmation_open(false);
                return;
            }
        };
        let selected = match active.list_movements() {
            Ok(movements) => movement_at_position(movements, position),
            Err(_) => {
                w.set_delete_status("No se pudo leer el movimiento".into());
                return;
            }
        };
        let Some(selected) = selected else {
            w.set_delete_status("No existe un movimiento con ese número".into());
            w.set_delete_confirmation_open(false);
            return;
        };
        match remove_movement(active, &selected.id) {
            Ok(true) => {
                refresh_dashboard(&w, active);
                w.set_delete_confirmation_open(false);
            }
            Ok(false) | Err(_) => w.set_delete_status("No se pudo eliminar el movimiento".into()),
        }
    });
    let planning_ledger = Rc::clone(&ledger);
    let weak = window.as_weak();
    window.on_save_budget(move |category, limit, year, month| {
        let Some(w) = weak.upgrade() else { return };
        let mut active = planning_ledger.borrow_mut();
        let Some(active) = active.as_mut() else {
            w.set_planning_status("Desbloquea primero tu espacio local".into());
            return;
        };
        match build_budget(&category, &limit, &year, &month) {
            Ok(budget) => match record_category_budget(active, budget) {
                Ok(()) => {
                    refresh_dashboard(&w, active);
                    w.set_budget_category_input("".into());
                    w.set_budget_limit_input("".into());
                }
                Err(_) => w.set_planning_status("No se pudo guardar el presupuesto".into()),
            },
            Err(_) => w.set_planning_status("Completa categoría y límite válido".into()),
        }
    });
    let goal_ledger = Rc::clone(&ledger);
    let weak = window.as_weak();
    window.on_save_goal(move |name, target, saved| {
        let Some(w) = weak.upgrade() else { return };
        let mut active = goal_ledger.borrow_mut();
        let Some(active) = active.as_mut() else {
            w.set_planning_status("Desbloquea primero tu espacio local".into());
            return;
        };
        match build_savings_goal(&name, &target, &saved) {
            Ok(goal) => match record_savings_goal(active, goal) {
                Ok(()) => {
                    refresh_dashboard(&w, active);
                    w.set_goal_name_input("".into());
                    w.set_goal_target_input("".into());
                    w.set_goal_saved_input("0".into());
                }
                Err(_) => w.set_planning_status("No se pudo guardar el objetivo".into()),
            },
            Err(error) => w.set_planning_status(error.into()),
        }
    });
    let holdings_ledger = Rc::clone(&ledger);
    let weak = window.as_weak();
    window.on_save_holding(move |account, name, symbol, invested, value, date| {
        let Some(w) = weak.upgrade() else { return };
        let mut active = holdings_ledger.borrow_mut();
        let Some(active) = active.as_mut() else {
            w.set_holding_status("Desbloquea primero tu espacio local".into());
            return;
        };
        match build_holding(&account, &name, &symbol, &invested, &value, &date) {
            Ok(holding) => match record_portfolio_holding(active, holding) {
                Ok(()) => {
                    refresh_dashboard(&w, active);
                    w.set_holding_name_input("".into());
                    w.set_holding_symbol_input("".into());
                    w.set_holding_invested_input("0".into());
                    w.set_holding_value_input("0".into());
                }
                Err(_) => w.set_holding_status("No se pudo guardar la posición".into()),
            },
            Err(message) => w.set_holding_status(message.into()),
        }
    });
    let offers_ledger = Rc::clone(&ledger);
    let weak = window.as_weak();
    window.on_save_bank_offer(move |institution, product, fee, notes, source, date| {
        let Some(w) = weak.upgrade() else { return };
        let mut active = offers_ledger.borrow_mut();
        let Some(active) = active.as_mut() else {
            w.set_holding_status("Desbloquea primero tu espacio local".into());
            return;
        };
        match build_bank_offer(&institution, &product, &fee, &notes, &source, &date) {
            Ok(offer) => match record_bank_offer(active, offer) {
                Ok(()) => {
                    refresh_dashboard(&w, active);
                    w.set_bank_institution_input("".into());
                    w.set_bank_product_input("".into());
                    w.set_bank_fee_input("0".into());
                    w.set_bank_notes_input("".into());
                    w.set_bank_source_input("".into());
                }
                Err(_) => w.set_holding_status("No se pudo guardar la comparación".into()),
            },
            Err(message) => w.set_holding_status(message.into()),
        }
    });
    let subscription_ledger = Rc::clone(&ledger);
    let weak = window.as_weak();
    window.on_save_subscription(move |frequency, name, amount, date| {
        let Some(w) = weak.upgrade() else { return };
        let mut active = subscription_ledger.borrow_mut();
        let Some(active) = active.as_mut() else {
            w.set_subscription_status("Desbloquea primero tu espacio local".into());
            return;
        };
        match build_subscription(frequency, &name, &amount, &date) {
            Ok(subscription) => match record_subscription(active, subscription) {
                Ok(()) => {
                    refresh_dashboard(&w, active);
                    w.set_subscription_name_input("".into());
                    w.set_subscription_amount_input("".into());
                    w.set_subscription_form_open(false);
                }
                Err(_) => w.set_subscription_status("No se pudo guardar la suscripción".into()),
            },
            Err(error) => w.set_subscription_status(error.into()),
        }
    });
    let unlock_ledger = Rc::clone(&ledger);
    let weak = window.as_weak();
    window.on_unlock(move |passphrase, confirmation| {
        let Some(w) = weak.upgrade() else { return };
        if passphrase.is_empty() {
            w.set_unlock_status("Introduce una contraseña".into());
            return;
        }
        let path = database_path();
        let is_new = !path.exists();
        if is_new && passphrase != confirmation {
            w.set_unlock_status("Las contraseñas no coinciden".into());
            return;
        }
        if let Some(directory) = path.parent()
            && fs::create_dir_all(directory).is_err()
        {
            w.set_unlock_status("No se pudo preparar el directorio local".into());
            return;
        }
        match SqlCipherLedger::open(path, &passphrase) {
            Ok(opened) => {
                *unlock_ledger.borrow_mut() = Some(opened);
                if let Some(active) = unlock_ledger.borrow().as_ref() {
                    refresh_dashboard(&w, active);
                }
                w.set_unlock_input("".into());
                w.set_unlock_confirmation("".into());
                w.set_unlock_status("".into());
                w.set_is_unlocked(true);
                w.set_unlock_open(false);
            }
            Err(SqlCipherLedgerError::EmptyKey) => {
                w.set_unlock_status("Introduce una contraseña".into())
            }
            Err(_) => w.set_unlock_status(
                "No se pudo desbloquear la base. Comprueba la contraseña.".into(),
            ),
        }
    });
    let weak = window.as_weak();
    window.on_save_movement(move |kind, account, category, description, amount, date| {
        let Some(w) = weak.upgrade() else { return };
        let mut active = ledger.borrow_mut();
        let Some(active) = active.as_mut() else {
            w.set_form_status("Desbloquea primero tu espacio local".into());
            return;
        };
        let editing_id = w.get_editing_movement_id();
        match build_movement_with_id(
            &editing_id,
            kind,
            &account,
            &category,
            &description,
            &amount,
            &date,
        ) {
            Ok(movement) => match if editing_id.is_empty() {
                record_movement(active, movement)
            } else {
                revise_movement(active, movement).map(|_| ())
            } {
                Ok(()) => {
                    refresh_dashboard(&w, active);
                    w.set_category_input("".into());
                    w.set_description_input("".into());
                    w.set_amount_input("".into());
                    w.set_editing_movement_id("".into());
                    w.set_form_open(false);
                }
                Err(_) => w.set_form_status("No se pudo guardar el movimiento local".into()),
            },
            Err(error) => w.set_form_status(error.into()),
        }
    });
    window.run()
}

fn build_movement(
    kind: i32,
    account: &str,
    category: &str,
    description: &str,
    amount: &str,
    date: &str,
) -> Result<Movement, String> {
    let kind = if kind == 1 {
        MovementKind::Income
    } else {
        MovementKind::Expense
    };
    Movement::new(
        movement_id(),
        account.into(),
        category.into(),
        description.into(),
        kind,
        parse_amount(amount)?,
        parse_date(date)?,
    )
    .map_err(|_| "Completa categoría y descripción".into())
}

fn parse_movement_position(input: &str) -> Result<usize, ()> {
    input
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(())
}

fn movement_at_position(movements: Vec<Movement>, position: usize) -> Option<Movement> {
    movements.into_iter().nth(position.saturating_sub(1))
}

fn build_movement_with_id(
    id: &str,
    kind: i32,
    account: &str,
    category: &str,
    description: &str,
    amount: &str,
    date: &str,
) -> Result<Movement, String> {
    let mut movement = build_movement(kind, account, category, description, amount, date)?;
    if !id.is_empty() {
        movement.id = id.into();
    }
    Ok(movement)
}

fn build_subscription(
    frequency: i32,
    name: &str,
    amount: &str,
    date: &str,
) -> Result<Subscription, String> {
    Subscription::new(
        subscription_id(),
        "general".into(),
        name.into(),
        parse_amount(amount)?,
        if frequency == 1 {
            BillingFrequency::Yearly
        } else {
            BillingFrequency::Monthly
        },
        parse_date(date)?,
    )
    .map_err(|_| "Completa el nombre de la suscripción".into())
}

fn build_budget(
    category: &str,
    limit: &str,
    year: &str,
    month: &str,
) -> Result<CategoryBudget, String> {
    CategoryBudget::new(
        budget_id(),
        category.into(),
        year.parse().map_err(|_| "Año no válido")?,
        month.parse().map_err(|_| "Mes no válido")?,
        parse_amount(limit)?,
    )
    .map_err(|_| "Completa categoría y límite válido".into())
}

fn build_savings_goal(name: &str, target: &str, saved: &str) -> Result<SavingsGoal, String> {
    SavingsGoal::new(
        goal_id(),
        name.into(),
        parse_amount(target)?,
        parse_non_negative_amount(saved)?,
    )
    .map_err(|_| "Completa nombre y cantidades válidas".into())
}

fn build_holding(
    account: &str,
    name: &str,
    symbol: &str,
    invested: &str,
    value: &str,
    date: &str,
) -> Result<PortfolioHolding, String> {
    PortfolioHolding::new(
        format!("p-{}", timestamp_nanos()),
        account.into(),
        name.into(),
        symbol.into(),
        parse_non_negative_amount(invested)?,
        parse_non_negative_amount(value)?,
        parse_date(date)?,
    )
    .map_err(|_| "Completa posición, importes y fecha válidos".into())
}

fn build_bank_offer(
    institution: &str,
    product: &str,
    fee: &str,
    notes: &str,
    source: &str,
    date: &str,
) -> Result<BankOffer, String> {
    BankOffer::new(
        format!("o-{}", timestamp_nanos()),
        institution.into(),
        product.into(),
        parse_non_negative_amount(fee)?,
        notes.into(),
        source.into(),
        parse_date(date)?,
    )
    .map_err(|_| "Completa datos, fecha y URL oficial https:// válidos".into())
}

fn refresh_dashboard(window: &AppWindow, ledger: &SqlCipherLedger) {
    if let Ok(Ok(summary)) = calculate_current_summary(ledger) {
        window.set_income_value(money(summary.income).into());
        window.set_expense_value(money(summary.expenses).into());
        window.set_balance_value(money(summary.balance().unwrap_or(Money::ZERO)).into());
    }
    if let Ok(movements) = ledger.list_movements() {
        let recent = movements.first().map_or_else(
            || "Aún no hay movimientos. Añade el primero cuando estés listo.".to_owned(),
            |movement| {
                format!(
                    "Último movimiento: {} · {}",
                    movement.description,
                    money(movement.amount)
                )
            },
        );
        window.set_recent_text(recent.into());
        let list = movements
            .iter()
            .enumerate()
            .map(|(index, movement)| {
                let kind = match movement.kind {
                    MovementKind::Income => "Ingreso",
                    MovementKind::Expense => "Gasto",
                };
                format!(
                    "{}. {} · {} · {} · {} · {}",
                    index + 1,
                    movement.occurred_on.to_iso8601(),
                    kind,
                    movement.category,
                    movement.description,
                    money(movement.amount)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        window.set_movement_list_text(if list.is_empty() {
            "Todavía no hay movimientos guardados.".into()
        } else {
            list.into()
        });
        let mut expenses_by_category = BTreeMap::<String, Money>::new();
        for movement in &movements {
            if movement.kind == MovementKind::Expense {
                let total = expenses_by_category
                    .entry(movement.category.clone())
                    .or_insert(Money::ZERO);
                *total = total.checked_add(movement.amount).unwrap_or(*total);
            }
        }
        let analysis = expenses_by_category
            .iter()
            .map(|(category, total)| format!("{category}: {}", money(*total)))
            .collect::<Vec<_>>()
            .join("\n");
        window.set_category_analysis_text(if analysis.is_empty() {
            "Aún no hay gastos por categoría.".into()
        } else {
            analysis.into()
        });
    }
    if let Ok(subscriptions) = ledger.list_subscriptions() {
        let list = subscriptions
            .iter()
            .map(|subscription| {
                let frequency = match subscription.frequency {
                    BillingFrequency::Monthly => "Mensual",
                    BillingFrequency::Yearly => "Anual",
                };
                format!(
                    "{} · {} · {} · {}",
                    subscription.next_charge.to_iso8601(),
                    frequency,
                    subscription.name,
                    money(subscription.amount)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        window.set_subscription_list_text(if list.is_empty() {
            "No hay suscripciones guardadas.".into()
        } else {
            list.into()
        });
    }
    if let Ok(budgets) = ledger.list_category_budgets() {
        let movements = ledger.list_movements().unwrap_or_default();
        let list = budgets
            .iter()
            .filter_map(|budget| {
                budget.progress(&movements).ok().map(|progress| {
                    format!(
                        "{} · {} / {}",
                        budget.category,
                        money(progress.spent),
                        money(progress.limit)
                    )
                })
            })
            .collect::<Vec<_>>()
            .join("\n");
        window.set_budget_list_text(if list.is_empty() {
            "No hay presupuestos definidos.".into()
        } else {
            list.into()
        });
    }
    if let Ok(goals) = ledger.list_savings_goals() {
        let list = goals
            .iter()
            .map(|goal| {
                format!(
                    "{} · {} / {}",
                    goal.name,
                    money(goal.saved),
                    money(goal.target)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        window.set_goal_list_text(if list.is_empty() {
            "No hay objetivos de ahorro definidos.".into()
        } else {
            list.into()
        });
    }
    if let Ok(holdings) = ledger.list_portfolio_holdings() {
        let list = holdings
            .iter()
            .map(|holding| {
                format!(
                    "{} · {} · {} · valor {} · resultado {}",
                    holding.account_id,
                    holding.name,
                    holding.symbol,
                    money(holding.current_value),
                    holding.result().map(money).unwrap_or_else(|| "—".into()),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        window.set_holding_list_text(if list.is_empty() {
            "No hay posiciones anotadas.".into()
        } else {
            list.into()
        });
    }
    if let Ok(offers) = ledger.list_bank_offers() {
        let list = offers
            .iter()
            .map(|offer| {
                format!(
                    "{} · {} · {} / mes · consultado {}\n{}\n{}",
                    offer.institution,
                    offer.product,
                    money(offer.monthly_fee),
                    offer.checked_on.to_iso8601(),
                    offer.notes,
                    offer.official_source,
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        window.set_bank_list_text(if list.is_empty() {
            "No hay comparativas anotadas.".into()
        } else {
            list.into()
        });
    }
}

fn movement_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("m-{nanos}")
}

fn subscription_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("s-{nanos}")
}

fn budget_id() -> String {
    format!("b-{}", timestamp_nanos())
}
fn goal_id() -> String {
    format!("g-{}", timestamp_nanos())
}
fn timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn database_path() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
    } else if let Some(path) = std::env::var_os("XDG_DATA_HOME") {
        PathBuf::from(path)
    } else {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join(".local/share")
    };
    let saldo_nube = base.join("saldo-nube").join("finanzas.db");
    let legacy = base.join("finanzas-con-snoopy").join("finanzas.db");
    if legacy.exists() { legacy } else { saldo_nube }
}

fn backup_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    database_path()
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("backups")
        .join(format!("finanzas-backup-{nanos}.db"))
}
fn parse_amount(input: &str) -> Result<Money, String> {
    let (whole, fraction) = input.trim().split_once(',').unwrap_or((input.trim(), ""));
    let euros: i64 = whole
        .parse()
        .map_err(|_| "Usa un importe como 12,50".to_string())?;
    let cents = match fraction.len() {
        0 => 0,
        1 => {
            fraction
                .parse::<i64>()
                .map_err(|_| "Importe no válido".to_string())?
                * 10
        }
        2 => fraction
            .parse()
            .map_err(|_| "Importe no válido".to_string())?,
        _ => return Err("Máximo dos decimales".into()),
    };
    let total = euros
        .checked_mul(100)
        .and_then(|n| n.checked_add(cents))
        .ok_or("Importe demasiado grande")?;
    if total <= 0 {
        Err("El importe debe ser mayor que cero".into())
    } else {
        Ok(Money::from_cents(total))
    }
}

fn parse_non_negative_amount(input: &str) -> Result<Money, String> {
    if input.trim() == "0" || input.trim() == "0,00" {
        Ok(Money::ZERO)
    } else {
        parse_amount(input)
    }
}

fn parse_date(input: &str) -> Result<LocalDate, String> {
    let p: Vec<_> = input.split('-').collect();
    if p.len() != 3 {
        return Err("Usa la fecha AAAA-MM-DD".into());
    }
    LocalDate::new(
        p[0].parse().map_err(|_| "Fecha no válida")?,
        p[1].parse().map_err(|_| "Fecha no válida")?,
        p[2].parse().map_err(|_| "Fecha no válida")?,
    )
    .map_err(|_| "Fecha no válida".into())
}

fn money(value: Money) -> String {
    format!("{} €", money_input(value))
}

fn money_input(value: Money) -> String {
    let cents = value.cents();
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.unsigned_abs();
    format!("{sign}{},{:02}", abs / 100, abs % 100)
}

#[cfg(test)]
mod tests {
    use super::{Money, money, money_input, parse_amount, parse_movement_position};

    #[test]
    fn movement_position_is_one_based_and_rejects_invalid_input() {
        assert_eq!(parse_movement_position("1"), Ok(1));
        assert_eq!(parse_movement_position(" 12 "), Ok(12));
        assert!(parse_movement_position("0").is_err());
        assert!(parse_movement_position("dos").is_err());
    }

    #[test]
    fn money_pads_cents_to_two_digits() {
        assert_eq!(money(Money::from_cents(1205)), "12,05 €");
        assert_eq!(money(Money::from_cents(1200)), "12,00 €");
        assert_eq!(money(Money::from_cents(5)), "0,05 €");
        assert_eq!(money(Money::from_cents(-50)), "-0,50 €");
        assert_eq!(money(Money::from_cents(-1205)), "-12,05 €");
    }

    #[test]
    fn money_input_keeps_sign_and_two_decimals() {
        assert_eq!(money_input(Money::from_cents(1205)), "12,05");
        assert_eq!(money_input(Money::from_cents(1200)), "12,00");
        assert_eq!(money_input(Money::from_cents(-50)), "-0,50");
    }

    #[test]
    fn editing_an_amount_does_not_change_it() {
        for cents in 1..=5_000 {
            let shown = money_input(Money::from_cents(cents));
            let parsed = parse_amount(&shown).expect("el importe mostrado debe poder leerse");
            assert_eq!(parsed.cents(), cents, "importe {shown}");
        }
    }
}
