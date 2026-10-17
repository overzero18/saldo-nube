//! Reglas financieras puras, independientes de UI, archivos y red.

/// Cantidad monetaria expresada en céntimos para evitar errores de punto flotante.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Money {
    cents: i64,
}

impl Money {
    pub const ZERO: Self = Self { cents: 0 };

    pub const fn from_cents(cents: i64) -> Self {
        Self { cents }
    }

    pub const fn cents(self) -> i64 {
        self.cents
    }

    pub const fn is_positive(self) -> bool {
        self.cents > 0
    }

    pub const fn is_non_negative(self) -> bool {
        self.cents >= 0
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.cents.checked_add(other.cents).map(Self::from_cents)
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.cents.checked_sub(other.cents).map(Self::from_cents)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    EmptyField(&'static str),
    InvalidAmount,
    InvalidDate,
    ArithmeticOverflow,
    InvalidOfficialSource,
}

/// Posición anotada manualmente. No consulta cotizaciones ni emite recomendaciones.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortfolioHolding {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub symbol: String,
    pub invested: Money,
    pub current_value: Money,
    pub updated_on: LocalDate,
}

impl PortfolioHolding {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        account_id: String,
        name: String,
        symbol: String,
        invested: Money,
        current_value: Money,
        updated_on: LocalDate,
    ) -> Result<Self, DomainError> {
        validate_text("id", &id)?;
        validate_text("cuenta", &account_id)?;
        validate_text("nombre del activo", &name)?;
        validate_text("símbolo", &symbol)?;
        if !invested.is_non_negative() || !current_value.is_non_negative() {
            return Err(DomainError::InvalidAmount);
        }
        Ok(Self {
            id,
            account_id,
            name,
            symbol,
            invested,
            current_value,
            updated_on,
        })
    }

    pub fn result(&self) -> Option<Money> {
        self.current_value.checked_sub(self.invested)
    }
}

/// Comparación anotada por la persona usuaria con enlace oficial visible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankOffer {
    pub id: String,
    pub institution: String,
    pub product: String,
    pub monthly_fee: Money,
    pub notes: String,
    pub official_source: String,
    pub checked_on: LocalDate,
}

impl BankOffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        institution: String,
        product: String,
        monthly_fee: Money,
        notes: String,
        official_source: String,
        checked_on: LocalDate,
    ) -> Result<Self, DomainError> {
        validate_text("id", &id)?;
        validate_text("entidad", &institution)?;
        validate_text("producto", &product)?;
        validate_text("notas", &notes)?;
        if !monthly_fee.is_non_negative() {
            return Err(DomainError::InvalidAmount);
        }
        if !official_source.trim().starts_with("https://") {
            return Err(DomainError::InvalidOfficialSource);
        }
        Ok(Self {
            id,
            institution,
            product,
            monthly_fee,
            notes,
            official_source,
            checked_on,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LocalDate {
    year: u16,
    month: u8,
    day: u8,
}

impl LocalDate {
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, DomainError> {
        if year == 0
            || month == 0
            || month > 12
            || day == 0
            || day > Self::days_in_month(year, month)
        {
            return Err(DomainError::InvalidDate);
        }
        Ok(Self { year, month, day })
    }

    pub const fn year(self) -> u16 {
        self.year
    }

    pub const fn month(self) -> u8 {
        self.month
    }

    pub const fn day(self) -> u8 {
        self.day
    }

    /// Fecha local estable para persistencia, sin zona horaria ni hora.
    pub fn to_iso8601(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub fn from_iso8601(value: &str) -> Result<Self, DomainError> {
        let mut parts = value.split('-');
        let year = parts
            .next()
            .and_then(|part| part.parse().ok())
            .ok_or(DomainError::InvalidDate)?;
        let month = parts
            .next()
            .and_then(|part| part.parse().ok())
            .ok_or(DomainError::InvalidDate)?;
        let day = parts
            .next()
            .and_then(|part| part.parse().ok())
            .ok_or(DomainError::InvalidDate)?;
        if parts.next().is_some() {
            return Err(DomainError::InvalidDate);
        }
        Self::new(year, month, day)
    }

    const fn days_in_month(year: u16, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if (year.is_multiple_of(4) && !year.is_multiple_of(100))
                || year.is_multiple_of(400) =>
            {
                29
            }
            2 => 28,
            _ => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MovementKind {
    Income,
    Expense,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Movement {
    pub id: String,
    pub account_id: String,
    pub category: String,
    pub description: String,
    pub kind: MovementKind,
    pub amount: Money,
    pub occurred_on: LocalDate,
}

impl Movement {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        account_id: String,
        category: String,
        description: String,
        kind: MovementKind,
        amount: Money,
        occurred_on: LocalDate,
    ) -> Result<Self, DomainError> {
        validate_text("id", &id)?;
        validate_text("cuenta", &account_id)?;
        validate_text("categoría", &category)?;
        validate_text("descripción", &description)?;
        if !amount.is_positive() {
            return Err(DomainError::InvalidAmount);
        }
        Ok(Self {
            id,
            account_id,
            category,
            description,
            kind,
            amount,
            occurred_on,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BillingFrequency {
    Monthly,
    Yearly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub amount: Money,
    pub frequency: BillingFrequency,
    pub next_charge: LocalDate,
}

impl Subscription {
    pub fn new(
        id: String,
        account_id: String,
        name: String,
        amount: Money,
        frequency: BillingFrequency,
        next_charge: LocalDate,
    ) -> Result<Self, DomainError> {
        validate_text("id", &id)?;
        validate_text("cuenta", &account_id)?;
        validate_text("nombre de suscripción", &name)?;
        if !amount.is_positive() {
            return Err(DomainError::InvalidAmount);
        }
        Ok(Self {
            id,
            account_id,
            name,
            amount,
            frequency,
            next_charge,
        })
    }
}

/// Resumen reproducible de ingresos, gastos y resultado neto.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BudgetSummary {
    pub income: Money,
    pub expenses: Money,
}

/// Límite de gasto de una categoría para un mes local concreto.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CategoryBudget {
    pub id: String,
    pub category: String,
    pub year: u16,
    pub month: u8,
    pub limit: Money,
}

impl CategoryBudget {
    pub fn new(
        id: String,
        category: String,
        year: u16,
        month: u8,
        limit: Money,
    ) -> Result<Self, DomainError> {
        validate_text("id", &id)?;
        validate_text("categoría", &category)?;
        if year == 0 || month == 0 || month > 12 {
            return Err(DomainError::InvalidDate);
        }
        if !limit.is_positive() {
            return Err(DomainError::InvalidAmount);
        }
        Ok(Self {
            id,
            category,
            year,
            month,
            limit,
        })
    }

    pub fn progress(&self, movements: &[Movement]) -> Result<BudgetProgress, DomainError> {
        let spent = movements
            .iter()
            .filter(|movement| {
                movement.kind == MovementKind::Expense
                    && movement.category == self.category
                    && movement.occurred_on.year() == self.year
                    && movement.occurred_on.month() == self.month
            })
            .try_fold(Money::ZERO, |total, movement| {
                total
                    .checked_add(movement.amount)
                    .ok_or(DomainError::ArithmeticOverflow)
            })?;
        Ok(BudgetProgress {
            limit: self.limit,
            spent,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetProgress {
    pub limit: Money,
    pub spent: Money,
}

impl BudgetProgress {
    pub fn remaining(self) -> Option<Money> {
        self.limit.checked_sub(self.spent)
    }

    pub fn is_exceeded(self) -> bool {
        self.spent > self.limit
    }
}

/// Objetivo de ahorro manual; no representa una cuenta ni una recomendación financiera.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavingsGoal {
    pub id: String,
    pub name: String,
    pub target: Money,
    pub saved: Money,
}

impl SavingsGoal {
    pub fn new(id: String, name: String, target: Money, saved: Money) -> Result<Self, DomainError> {
        validate_text("id", &id)?;
        validate_text("nombre del objetivo", &name)?;
        if !target.is_positive() || !saved.is_non_negative() {
            return Err(DomainError::InvalidAmount);
        }
        Ok(Self {
            id,
            name,
            target,
            saved,
        })
    }

    pub fn remaining(&self) -> Option<Money> {
        self.target.checked_sub(self.saved)
    }

    pub fn is_complete(&self) -> bool {
        self.saved >= self.target
    }
}

impl BudgetSummary {
    pub fn from_movements(movements: &[Movement]) -> Result<Self, DomainError> {
        movements
            .iter()
            .try_fold(Self::default(), |mut summary, movement| {
                let total = match movement.kind {
                    MovementKind::Income => &mut summary.income,
                    MovementKind::Expense => &mut summary.expenses,
                };
                *total = total
                    .checked_add(movement.amount)
                    .ok_or(DomainError::ArithmeticOverflow)?;
                Ok(summary)
            })
    }

    pub fn balance(self) -> Option<Money> {
        self.income.checked_sub(self.expenses)
    }
}

fn validate_text(field: &'static str, value: &str) -> Result<(), DomainError> {
    if value.trim().is_empty() {
        Err(DomainError::EmptyField(field))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BankOffer, BillingFrequency, BudgetSummary, CategoryBudget, DomainError, LocalDate, Money,
        Movement, MovementKind, PortfolioHolding, SavingsGoal, Subscription,
    };

    fn date() -> LocalDate {
        LocalDate::new(2026, 10, 17).unwrap()
    }

    fn expense(amount: Money) -> Result<Movement, DomainError> {
        Movement::new(
            "m-1".into(),
            "cuenta-principal".into(),
            "hogar".into(),
            "electricidad".into(),
            MovementKind::Expense,
            amount,
            date(),
        )
    }

    #[test]
    fn money_is_calculated_in_exact_cents() {
        assert_eq!(
            Money::from_cents(199).checked_add(Money::from_cents(1)),
            Some(Money::from_cents(200))
        );
    }

    #[test]
    fn leap_day_is_valid_only_in_a_leap_year() {
        assert!(LocalDate::new(2028, 2, 29).is_ok());
        assert_eq!(LocalDate::new(2027, 2, 29), Err(DomainError::InvalidDate));
    }

    #[test]
    fn movement_requires_a_positive_amount() {
        assert_eq!(
            expense(Money::ZERO).unwrap_err(),
            DomainError::InvalidAmount
        );
    }

    #[test]
    fn budget_summary_groups_income_and_expenses() {
        let income = Movement::new(
            "m-2".into(),
            "cuenta-principal".into(),
            "nómina".into(),
            "nómina".into(),
            MovementKind::Income,
            Money::from_cents(150_000),
            date(),
        )
        .unwrap();
        let summary =
            BudgetSummary::from_movements(&[income, expense(Money::from_cents(42_500)).unwrap()])
                .unwrap();
        assert_eq!(summary.income, Money::from_cents(150_000));
        assert_eq!(summary.expenses, Money::from_cents(42_500));
        assert_eq!(summary.balance(), Some(Money::from_cents(107_500)));
    }

    #[test]
    fn subscription_needs_a_name() {
        let result = Subscription::new(
            "s-1".into(),
            "cuenta-principal".into(),
            " ".into(),
            Money::from_cents(999),
            BillingFrequency::Monthly,
            date(),
        );
        assert_eq!(
            result,
            Err(DomainError::EmptyField("nombre de suscripción"))
        );
    }

    #[test]
    fn category_budget_counts_only_matching_monthly_expenses() {
        let budget = CategoryBudget::new(
            "b-1".into(),
            "hogar".into(),
            2026,
            10,
            Money::from_cents(10_000),
        )
        .unwrap();
        let matching = expense(Money::from_cents(2_500)).unwrap();
        let other_month = Movement::new(
            "m-3".into(),
            "cuenta-principal".into(),
            "hogar".into(),
            "noviembre".into(),
            MovementKind::Expense,
            Money::from_cents(1_000),
            LocalDate::new(2026, 11, 1).unwrap(),
        )
        .unwrap();

        let progress = budget.progress(&[matching, other_month]).unwrap();
        assert_eq!(progress.spent, Money::from_cents(2_500));
        assert_eq!(progress.remaining(), Some(Money::from_cents(7_500)));
        assert!(!progress.is_exceeded());
    }

    #[test]
    fn savings_goal_can_be_completed_without_exceeding_target_rules() {
        let goal = SavingsGoal::new(
            "g-1".into(),
            "colchón".into(),
            Money::from_cents(100_000),
            Money::from_cents(120_000),
        )
        .unwrap();
        assert!(goal.is_complete());
        assert_eq!(goal.remaining(), Some(Money::from_cents(-20_000)));
    }

    #[test]
    fn portfolio_holding_tracks_a_manual_result_without_market_data() {
        let holding = PortfolioHolding::new(
            "p-1".into(),
            "trade republic".into(),
            "Fondo índice".into(),
            "INDEX".into(),
            Money::from_cents(10_000),
            Money::from_cents(10_500),
            date(),
        )
        .unwrap();
        assert_eq!(holding.result(), Some(Money::from_cents(500)));
    }

    #[test]
    fn bank_offer_requires_an_official_https_source() {
        let result = BankOffer::new(
            "o-1".into(),
            "Entidad".into(),
            "Cuenta".into(),
            Money::ZERO,
            "Sin comisión de mantenimiento según la ficha.".into(),
            "http://ejemplo.test".into(),
            date(),
        );
        assert_eq!(result, Err(DomainError::InvalidOfficialSource));
    }
}
