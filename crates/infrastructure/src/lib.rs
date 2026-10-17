//! Adaptadores de infraestructura local. No añade red, telemetría ni servicios.

use finanzas_application::LedgerRepository;
use finanzas_domain::{
    BankOffer, BillingFrequency, CategoryBudget, DomainError, LocalDate, Money, Movement,
    MovementKind, PortfolioHolding, SavingsGoal, Subscription,
};
use rusqlite::backup::Backup;
use rusqlite::{Connection, OptionalExtension, params};
use std::{path::Path, time::Duration};

#[derive(Debug)]
pub enum SqlCipherLedgerError {
    EmptyKey,
    BackupAlreadyExists,
    Database(rusqlite::Error),
    InvalidStoredData(DomainError),
    UnknownMovementKind(i64),
    UnknownBillingFrequency(String),
}

impl From<rusqlite::Error> for SqlCipherLedgerError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

/// Base de datos SQLCipher embebida. La contraseña nunca se conserva en esta estructura.
pub struct SqlCipherLedger {
    connection: Connection,
}

impl SqlCipherLedger {
    /// Abre o crea una base cifrada y aplica las migraciones pendientes de forma transaccional.
    pub fn open(path: impl AsRef<Path>, passphrase: &str) -> Result<Self, SqlCipherLedgerError> {
        if passphrase.is_empty() {
            return Err(SqlCipherLedgerError::EmptyKey);
        }

        let mut connection = Connection::open(path)?;
        configure_cipher(&connection, passphrase)?;
        migrate(&mut connection)?;
        Ok(Self { connection })
    }

    /// Genera una copia coherente y cifrada sin copiar el fichero abierto directamente.
    /// La contraseña se recibe solo para esta operación y no se conserva.
    pub fn create_encrypted_backup(
        &self,
        path: impl AsRef<Path>,
        passphrase: &str,
    ) -> Result<(), SqlCipherLedgerError> {
        if passphrase.is_empty() {
            return Err(SqlCipherLedgerError::EmptyKey);
        }
        let path = path.as_ref();
        if path.exists() {
            return Err(SqlCipherLedgerError::BackupAlreadyExists);
        }
        let mut destination = Connection::open(path)?;
        configure_cipher(&destination, passphrase)?;
        Backup::new(&self.connection, &mut destination)?.run_to_completion(
            64,
            Duration::from_millis(1),
            None,
        )?;
        destination.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;
        Ok(())
    }

    #[cfg(test)]
    fn open_in_memory(passphrase: &str) -> Result<Self, SqlCipherLedgerError> {
        if passphrase.is_empty() {
            return Err(SqlCipherLedgerError::EmptyKey);
        }
        let mut connection = Connection::open_in_memory()?;
        configure_cipher(&connection, passphrase)?;
        migrate(&mut connection)?;
        Ok(Self { connection })
    }
}

fn configure_cipher(connection: &Connection, passphrase: &str) -> Result<(), rusqlite::Error> {
    // rusqlite enlaza el valor sin construir SQL: no se interpolan contraseñas en sentencias.
    connection.pragma_update(None, "key", passphrase)?;
    connection.execute_batch(
        "PRAGMA cipher_memory_security = ON;
         PRAGMA foreign_keys = ON;",
    )?;
    // Fuerza una lectura para detectar inmediatamente una contraseña incorrecta.
    connection.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;
    Ok(())
}

fn migrate(connection: &mut Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
             version INTEGER PRIMARY KEY NOT NULL
         );",
    )?;
    let migration_one_applied: Option<i64> = connection
        .query_row(
            "SELECT version FROM schema_migrations WHERE version = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if migration_one_applied.is_none() {
        let transaction = connection.transaction()?;
        transaction.execute_batch(
            "CREATE TABLE accounts (
             id TEXT PRIMARY KEY NOT NULL,
             label TEXT NOT NULL
         );
         CREATE TABLE movements (
             id TEXT PRIMARY KEY NOT NULL,
             account_id TEXT NOT NULL,
             category TEXT NOT NULL,
             description TEXT NOT NULL,
             kind INTEGER NOT NULL CHECK(kind IN (0, 1)),
             amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
             occurred_on TEXT NOT NULL,
             FOREIGN KEY(account_id) REFERENCES accounts(id)
         );
         CREATE TABLE subscriptions (
             id TEXT PRIMARY KEY NOT NULL,
             account_id TEXT NOT NULL,
             name TEXT NOT NULL,
             amount_cents INTEGER NOT NULL CHECK(amount_cents > 0),
             frequency TEXT NOT NULL CHECK(frequency IN ('monthly', 'yearly')),
             next_charge TEXT NOT NULL,
             FOREIGN KEY(account_id) REFERENCES accounts(id)
         );
         CREATE TABLE preferences (
             key TEXT PRIMARY KEY NOT NULL,
             value TEXT NOT NULL
         );
         INSERT INTO schema_migrations(version) VALUES (1);",
        )?;
        transaction.commit()?;
    }

    let migration_two_applied: Option<i64> = connection
        .query_row(
            "SELECT version FROM schema_migrations WHERE version = 2",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if migration_two_applied.is_none() {
        let transaction = connection.transaction()?;
        transaction.execute_batch(
            "CREATE TABLE category_budgets (
                 id TEXT PRIMARY KEY NOT NULL,
                 category TEXT NOT NULL,
                 year INTEGER NOT NULL,
                 month INTEGER NOT NULL CHECK(month BETWEEN 1 AND 12),
                 limit_cents INTEGER NOT NULL CHECK(limit_cents > 0)
             );
             CREATE TABLE savings_goals (
                 id TEXT PRIMARY KEY NOT NULL,
                 name TEXT NOT NULL,
                 target_cents INTEGER NOT NULL CHECK(target_cents > 0),
                 saved_cents INTEGER NOT NULL CHECK(saved_cents >= 0)
             );
             INSERT INTO schema_migrations(version) VALUES (2);",
        )?;
        transaction.commit()?;
    }
    let migration_three_applied: Option<i64> = connection
        .query_row(
            "SELECT version FROM schema_migrations WHERE version = 3",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if migration_three_applied.is_none() {
        let transaction = connection.transaction()?;
        transaction.execute_batch(
            "CREATE TABLE portfolio_holdings (
                 id TEXT PRIMARY KEY NOT NULL,
                 account_id TEXT NOT NULL,
                 name TEXT NOT NULL,
                 symbol TEXT NOT NULL,
                 invested_cents INTEGER NOT NULL CHECK(invested_cents >= 0),
                 current_value_cents INTEGER NOT NULL CHECK(current_value_cents >= 0),
                 updated_on TEXT NOT NULL,
                 FOREIGN KEY(account_id) REFERENCES accounts(id)
             );
             CREATE TABLE bank_offers (
                 id TEXT PRIMARY KEY NOT NULL,
                 institution TEXT NOT NULL,
                 product TEXT NOT NULL,
                 monthly_fee_cents INTEGER NOT NULL CHECK(monthly_fee_cents >= 0),
                 notes TEXT NOT NULL,
                 official_source TEXT NOT NULL,
                 checked_on TEXT NOT NULL
             );
             INSERT INTO schema_migrations(version) VALUES (3);",
        )?;
        transaction.commit()?;
    }
    Ok(())
}

impl LedgerRepository for SqlCipherLedger {
    type Error = SqlCipherLedgerError;

    fn save_movement(&mut self, movement: Movement) -> Result<(), Self::Error> {
        self.connection.execute(
            "INSERT OR IGNORE INTO accounts(id, label) VALUES (?1, ?1)",
            [&movement.account_id],
        )?;
        self.connection.execute(
            "INSERT INTO movements(id, account_id, category, description, kind, amount_cents, occurred_on)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                movement.id,
                movement.account_id,
                movement.category,
                movement.description,
                movement_kind_to_db(movement.kind),
                movement.amount.cents(),
                movement.occurred_on.to_iso8601(),
            ],
        )?;
        Ok(())
    }

    fn list_movements(&self) -> Result<Vec<Movement>, Self::Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, category, description, kind, amount_cents, occurred_on
             FROM movements ORDER BY occurred_on DESC, id DESC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;
        rows.map(|row| {
            let (id, account_id, category, description, kind, amount_cents, occurred_on) = row?;
            Movement::new(
                id,
                account_id,
                category,
                description,
                movement_kind_from_db(kind)?,
                Money::from_cents(amount_cents),
                LocalDate::from_iso8601(&occurred_on)
                    .map_err(SqlCipherLedgerError::InvalidStoredData)?,
            )
            .map_err(SqlCipherLedgerError::InvalidStoredData)
        })
        .collect()
    }

    fn delete_movement(&mut self, movement_id: &str) -> Result<bool, Self::Error> {
        Ok(self
            .connection
            .execute("DELETE FROM movements WHERE id = ?1", [movement_id])?
            == 1)
    }

    fn update_movement(&mut self, movement: Movement) -> Result<bool, Self::Error> {
        self.connection.execute(
            "INSERT OR IGNORE INTO accounts(id, label) VALUES (?1, ?1)",
            [&movement.account_id],
        )?;
        Ok(self.connection.execute(
            "UPDATE movements SET account_id = ?2, category = ?3, description = ?4, kind = ?5, amount_cents = ?6, occurred_on = ?7 WHERE id = ?1",
            params![movement.id, movement.account_id, movement.category, movement.description, movement_kind_to_db(movement.kind), movement.amount.cents(), movement.occurred_on.to_iso8601()],
        )? == 1)
    }

    fn save_subscription(&mut self, subscription: Subscription) -> Result<(), Self::Error> {
        self.connection.execute(
            "INSERT OR IGNORE INTO accounts(id, label) VALUES (?1, ?1)",
            [&subscription.account_id],
        )?;
        self.connection.execute(
            "INSERT INTO subscriptions(id, account_id, name, amount_cents, frequency, next_charge)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                subscription.id,
                subscription.account_id,
                subscription.name,
                subscription.amount.cents(),
                billing_frequency_to_db(subscription.frequency),
                subscription.next_charge.to_iso8601(),
            ],
        )?;
        Ok(())
    }

    fn list_subscriptions(&self) -> Result<Vec<Subscription>, Self::Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, name, amount_cents, frequency, next_charge
             FROM subscriptions ORDER BY next_charge ASC, id ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        rows.map(|row| {
            let (id, account_id, name, amount_cents, frequency, next_charge) = row?;
            Subscription::new(
                id,
                account_id,
                name,
                Money::from_cents(amount_cents),
                billing_frequency_from_db(frequency)?,
                LocalDate::from_iso8601(&next_charge)
                    .map_err(SqlCipherLedgerError::InvalidStoredData)?,
            )
            .map_err(SqlCipherLedgerError::InvalidStoredData)
        })
        .collect()
    }

    fn save_category_budget(&mut self, budget: CategoryBudget) -> Result<(), Self::Error> {
        self.connection.execute(
            "INSERT INTO category_budgets(id, category, year, month, limit_cents)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                budget.id,
                budget.category,
                budget.year,
                budget.month,
                budget.limit.cents()
            ],
        )?;
        Ok(())
    }

    fn list_category_budgets(&self) -> Result<Vec<CategoryBudget>, Self::Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, category, year, month, limit_cents FROM category_budgets ORDER BY year DESC, month DESC, category ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u16>(2)?,
                row.get::<_, u8>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;
        rows.map(|row| {
            let (id, category, year, month, limit) = row?;
            CategoryBudget::new(id, category, year, month, Money::from_cents(limit))
                .map_err(SqlCipherLedgerError::InvalidStoredData)
        })
        .collect()
    }

    fn save_savings_goal(&mut self, goal: SavingsGoal) -> Result<(), Self::Error> {
        self.connection.execute(
            "INSERT INTO savings_goals(id, name, target_cents, saved_cents) VALUES (?1, ?2, ?3, ?4)",
            params![goal.id, goal.name, goal.target.cents(), goal.saved.cents()],
        )?;
        Ok(())
    }

    fn list_savings_goals(&self) -> Result<Vec<SavingsGoal>, Self::Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, target_cents, saved_cents FROM savings_goals ORDER BY name ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;
        rows.map(|row| {
            let (id, name, target, saved) = row?;
            SavingsGoal::new(
                id,
                name,
                Money::from_cents(target),
                Money::from_cents(saved),
            )
            .map_err(SqlCipherLedgerError::InvalidStoredData)
        })
        .collect()
    }

    fn save_portfolio_holding(&mut self, holding: PortfolioHolding) -> Result<(), Self::Error> {
        self.connection.execute(
            "INSERT OR IGNORE INTO accounts(id, label) VALUES (?1, ?1)",
            [&holding.account_id],
        )?;
        self.connection.execute(
            "INSERT INTO portfolio_holdings(id, account_id, name, symbol, invested_cents, current_value_cents, updated_on)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![holding.id, holding.account_id, holding.name, holding.symbol, holding.invested.cents(), holding.current_value.cents(), holding.updated_on.to_iso8601()],
        )?;
        Ok(())
    }

    fn list_portfolio_holdings(&self) -> Result<Vec<PortfolioHolding>, Self::Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, account_id, name, symbol, invested_cents, current_value_cents, updated_on FROM portfolio_holdings ORDER BY updated_on DESC, name ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;
        rows.map(|row| {
            let (id, account_id, name, symbol, invested, current_value, updated_on) = row?;
            PortfolioHolding::new(
                id,
                account_id,
                name,
                symbol,
                Money::from_cents(invested),
                Money::from_cents(current_value),
                LocalDate::from_iso8601(&updated_on)
                    .map_err(SqlCipherLedgerError::InvalidStoredData)?,
            )
            .map_err(SqlCipherLedgerError::InvalidStoredData)
        })
        .collect()
    }

    fn save_bank_offer(&mut self, offer: BankOffer) -> Result<(), Self::Error> {
        self.connection.execute(
            "INSERT INTO bank_offers(id, institution, product, monthly_fee_cents, notes, official_source, checked_on)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![offer.id, offer.institution, offer.product, offer.monthly_fee.cents(), offer.notes, offer.official_source, offer.checked_on.to_iso8601()],
        )?;
        Ok(())
    }

    fn list_bank_offers(&self) -> Result<Vec<BankOffer>, Self::Error> {
        let mut statement = self.connection.prepare(
            "SELECT id, institution, product, monthly_fee_cents, notes, official_source, checked_on FROM bank_offers ORDER BY checked_on DESC, institution ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;
        rows.map(|row| {
            let (id, institution, product, fee, notes, source, checked_on) = row?;
            BankOffer::new(
                id,
                institution,
                product,
                Money::from_cents(fee),
                notes,
                source,
                LocalDate::from_iso8601(&checked_on)
                    .map_err(SqlCipherLedgerError::InvalidStoredData)?,
            )
            .map_err(SqlCipherLedgerError::InvalidStoredData)
        })
        .collect()
    }
}

const fn movement_kind_to_db(kind: MovementKind) -> i64 {
    match kind {
        MovementKind::Expense => 0,
        MovementKind::Income => 1,
    }
}

fn movement_kind_from_db(value: i64) -> Result<MovementKind, SqlCipherLedgerError> {
    match value {
        0 => Ok(MovementKind::Expense),
        1 => Ok(MovementKind::Income),
        value => Err(SqlCipherLedgerError::UnknownMovementKind(value)),
    }
}

const fn billing_frequency_to_db(frequency: BillingFrequency) -> &'static str {
    match frequency {
        BillingFrequency::Monthly => "monthly",
        BillingFrequency::Yearly => "yearly",
    }
}

fn billing_frequency_from_db(value: String) -> Result<BillingFrequency, SqlCipherLedgerError> {
    match value.as_str() {
        "monthly" => Ok(BillingFrequency::Monthly),
        "yearly" => Ok(BillingFrequency::Yearly),
        _ => Err(SqlCipherLedgerError::UnknownBillingFrequency(value)),
    }
}

/// Repositorio temporal para pruebas y prototipos. No persiste datos.
#[derive(Default)]
pub struct InMemoryLedger {
    movements: Vec<Movement>,
    subscriptions: Vec<Subscription>,
    category_budgets: Vec<CategoryBudget>,
    savings_goals: Vec<SavingsGoal>,
    portfolio_holdings: Vec<PortfolioHolding>,
    bank_offers: Vec<BankOffer>,
}

impl LedgerRepository for InMemoryLedger {
    type Error = core::convert::Infallible;

    fn save_movement(&mut self, movement: Movement) -> Result<(), Self::Error> {
        self.movements.push(movement);
        Ok(())
    }

    fn list_movements(&self) -> Result<Vec<Movement>, Self::Error> {
        Ok(self.movements.clone())
    }

    fn delete_movement(&mut self, movement_id: &str) -> Result<bool, Self::Error> {
        let original_length = self.movements.len();
        self.movements.retain(|movement| movement.id != movement_id);
        Ok(self.movements.len() != original_length)
    }

    fn update_movement(&mut self, movement: Movement) -> Result<bool, Self::Error> {
        let Some(existing) = self
            .movements
            .iter_mut()
            .find(|item| item.id == movement.id)
        else {
            return Ok(false);
        };
        *existing = movement;
        Ok(true)
    }

    fn save_subscription(&mut self, subscription: Subscription) -> Result<(), Self::Error> {
        self.subscriptions.push(subscription);
        Ok(())
    }

    fn list_subscriptions(&self) -> Result<Vec<Subscription>, Self::Error> {
        Ok(self.subscriptions.clone())
    }

    fn save_category_budget(&mut self, budget: CategoryBudget) -> Result<(), Self::Error> {
        self.category_budgets.push(budget);
        Ok(())
    }

    fn list_category_budgets(&self) -> Result<Vec<CategoryBudget>, Self::Error> {
        Ok(self.category_budgets.clone())
    }

    fn save_savings_goal(&mut self, goal: SavingsGoal) -> Result<(), Self::Error> {
        self.savings_goals.push(goal);
        Ok(())
    }

    fn list_savings_goals(&self) -> Result<Vec<SavingsGoal>, Self::Error> {
        Ok(self.savings_goals.clone())
    }

    fn save_portfolio_holding(&mut self, holding: PortfolioHolding) -> Result<(), Self::Error> {
        self.portfolio_holdings.push(holding);
        Ok(())
    }

    fn list_portfolio_holdings(&self) -> Result<Vec<PortfolioHolding>, Self::Error> {
        Ok(self.portfolio_holdings.clone())
    }

    fn save_bank_offer(&mut self, offer: BankOffer) -> Result<(), Self::Error> {
        self.bank_offers.push(offer);
        Ok(())
    }

    fn list_bank_offers(&self) -> Result<Vec<BankOffer>, Self::Error> {
        Ok(self.bank_offers.clone())
    }
}

#[cfg(test)]
mod tests {
    use finanzas_application::{
        LedgerRepository, calculate_current_summary, record_movement, remove_movement,
    };
    use finanzas_domain::{LocalDate, Money, Movement, MovementKind};

    use super::{InMemoryLedger, SqlCipherLedger, SqlCipherLedgerError, configure_cipher, migrate};

    #[test]
    fn recorded_movements_are_available_to_the_summary_use_case() {
        let movement = Movement::new(
            "m-1".into(),
            "cuenta-principal".into(),
            "alimentación".into(),
            "compra".into(),
            MovementKind::Expense,
            Money::from_cents(2_350),
            LocalDate::new(2026, 10, 17).unwrap(),
        )
        .unwrap();
        let mut ledger = InMemoryLedger::default();

        record_movement(&mut ledger, movement).unwrap();
        let summary = calculate_current_summary(&ledger).unwrap().unwrap();

        assert_eq!(summary.income, Money::ZERO);
        assert_eq!(summary.expenses, Money::from_cents(2_350));
        assert_eq!(summary.balance(), Some(Money::from_cents(-2_350)));
    }

    #[test]
    fn sqlcipher_round_trip_keeps_data_local_and_readable_after_reopen() {
        let path = std::env::temp_dir().join(format!("finanzas-test-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let movement = Movement::new(
            "cipher-m-1".into(),
            "cuenta-principal".into(),
            "hogar".into(),
            "electricidad".into(),
            MovementKind::Expense,
            Money::from_cents(2_350),
            LocalDate::new(2026, 10, 17).unwrap(),
        )
        .unwrap();

        let mut ledger = SqlCipherLedger::open(&path, "clave-prueba-solo-tests").unwrap();
        record_movement(&mut ledger, movement.clone()).unwrap();
        drop(ledger);

        let reopened = SqlCipherLedger::open(&path, "clave-prueba-solo-tests").unwrap();
        assert_eq!(reopened.list_movements().unwrap(), vec![movement]);
        assert!(SqlCipherLedger::open(&path, "clave-incorrecta").is_err());
        drop(reopened);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn empty_key_is_rejected_before_opening_a_database() {
        assert!(matches!(
            SqlCipherLedger::open_in_memory(""),
            Err(SqlCipherLedgerError::EmptyKey)
        ));
    }

    #[test]
    fn encrypted_backup_can_be_opened_with_its_key() {
        let source_path =
            std::env::temp_dir().join(format!("finanzas-source-{}.db", std::process::id()));
        let backup_path =
            std::env::temp_dir().join(format!("finanzas-backup-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(&backup_path);
        let movement = Movement::new(
            "backup-m-1".into(),
            "cuenta-principal".into(),
            "hogar".into(),
            "internet".into(),
            MovementKind::Expense,
            Money::from_cents(3_000),
            LocalDate::new(2026, 10, 17).unwrap(),
        )
        .unwrap();
        let mut source = SqlCipherLedger::open(&source_path, "clave-copia-prueba").unwrap();
        record_movement(&mut source, movement.clone()).unwrap();

        source
            .create_encrypted_backup(&backup_path, "clave-copia-prueba")
            .unwrap();
        let backup = SqlCipherLedger::open(&backup_path, "clave-copia-prueba").unwrap();
        assert_eq!(backup.list_movements().unwrap(), vec![movement]);
        drop(backup);
        drop(source);
        let _ = std::fs::remove_file(source_path);
        let _ = std::fs::remove_file(backup_path);
    }

    #[test]
    fn subscriptions_are_persisted_in_the_encrypted_ledger() {
        let subscription = finanzas_domain::Subscription::new(
            "sub-1".into(),
            "cuenta-principal".into(),
            "música".into(),
            Money::from_cents(999),
            finanzas_domain::BillingFrequency::Monthly,
            LocalDate::new(2026, 11, 1).unwrap(),
        )
        .unwrap();
        let mut ledger = SqlCipherLedger::open_in_memory("clave-prueba").unwrap();

        finanzas_application::record_subscription(&mut ledger, subscription.clone()).unwrap();
        assert_eq!(ledger.list_subscriptions().unwrap(), vec![subscription]);
    }

    #[test]
    fn holdings_and_bank_comparisons_survive_encrypted_reopen() {
        let path =
            std::env::temp_dir().join(format!("finanzas-portfolio-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let holding = finanzas_domain::PortfolioHolding::new(
            "p-1".into(),
            "trade republic".into(),
            "Fondo índice".into(),
            "INDEX".into(),
            Money::from_cents(10_000),
            Money::from_cents(10_250),
            LocalDate::new(2026, 10, 17).unwrap(),
        )
        .unwrap();
        let offer = finanzas_domain::BankOffer::new(
            "o-1".into(),
            "Entidad de prueba".into(),
            "Cuenta local".into(),
            Money::ZERO,
            "Condiciones anotadas para la prueba".into(),
            "https://example.test/cuenta".into(),
            LocalDate::new(2026, 10, 17).unwrap(),
        )
        .unwrap();
        let mut ledger = SqlCipherLedger::open(&path, "clave-cartera-prueba").unwrap();
        finanzas_application::record_portfolio_holding(&mut ledger, holding.clone()).unwrap();
        finanzas_application::record_bank_offer(&mut ledger, offer.clone()).unwrap();
        drop(ledger);

        let reopened = SqlCipherLedger::open(&path, "clave-cartera-prueba").unwrap();
        assert_eq!(reopened.list_portfolio_holdings().unwrap(), vec![holding]);
        assert_eq!(reopened.list_bank_offers().unwrap(), vec![offer]);
        drop(reopened);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn migration_three_preserves_existing_movements() {
        let mut connection = rusqlite::Connection::open_in_memory().unwrap();
        configure_cipher(&connection, "clave-migracion-prueba").unwrap();
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL);
                 CREATE TABLE accounts (id TEXT PRIMARY KEY NOT NULL, label TEXT NOT NULL);
                 CREATE TABLE movements (
                     id TEXT PRIMARY KEY NOT NULL, account_id TEXT NOT NULL, category TEXT NOT NULL,
                     description TEXT NOT NULL, kind INTEGER NOT NULL, amount_cents INTEGER NOT NULL,
                     occurred_on TEXT NOT NULL
                 );
                 CREATE TABLE subscriptions (
                     id TEXT PRIMARY KEY NOT NULL, account_id TEXT NOT NULL, name TEXT NOT NULL,
                     amount_cents INTEGER NOT NULL, frequency TEXT NOT NULL, next_charge TEXT NOT NULL
                 );
                 CREATE TABLE preferences (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL);
                 CREATE TABLE category_budgets (
                     id TEXT PRIMARY KEY NOT NULL, category TEXT NOT NULL, year INTEGER NOT NULL,
                     month INTEGER NOT NULL, limit_cents INTEGER NOT NULL
                 );
                 CREATE TABLE savings_goals (
                     id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL, target_cents INTEGER NOT NULL,
                     saved_cents INTEGER NOT NULL
                 );
                 INSERT INTO schema_migrations(version) VALUES (1), (2);
                 INSERT INTO accounts(id, label) VALUES ('cuenta', 'cuenta');
                 INSERT INTO movements(id, account_id, category, description, kind, amount_cents, occurred_on)
                     VALUES ('existente', 'cuenta', 'hogar', 'luz', 0, 1234, '2026-10-17');",
            )
            .unwrap();

        migrate(&mut connection).unwrap();

        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM movements", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT version FROM schema_migrations WHERE version = 3",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            3
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM portfolio_holdings", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn deletion_affects_only_the_requested_movement() {
        let first = Movement::new(
            "keep".into(),
            "cuenta-principal".into(),
            "hogar".into(),
            "luz".into(),
            MovementKind::Expense,
            Money::from_cents(1_000),
            LocalDate::new(2026, 10, 1).unwrap(),
        )
        .unwrap();
        let second = Movement::new(
            "delete".into(),
            "cuenta-principal".into(),
            "hogar".into(),
            "agua".into(),
            MovementKind::Expense,
            Money::from_cents(500),
            LocalDate::new(2026, 10, 2).unwrap(),
        )
        .unwrap();
        let mut ledger = InMemoryLedger::default();
        record_movement(&mut ledger, first.clone()).unwrap();
        record_movement(&mut ledger, second).unwrap();

        assert!(remove_movement(&mut ledger, "delete").unwrap());
        assert_eq!(ledger.list_movements().unwrap(), vec![first]);
        assert!(!remove_movement(&mut ledger, "missing").unwrap());
    }
}
