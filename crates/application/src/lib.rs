//! Casos de uso. Esta capa depende del dominio, nunca de la interfaz gráfica.

use finanzas_domain::{
    BankOffer, BudgetSummary, CategoryBudget, Money, Movement, PortfolioHolding, SavingsGoal,
    Subscription,
};

pub trait LedgerRepository {
    type Error;

    fn save_movement(&mut self, movement: Movement) -> Result<(), Self::Error>;
    fn list_movements(&self) -> Result<Vec<Movement>, Self::Error>;
    fn delete_movement(&mut self, movement_id: &str) -> Result<bool, Self::Error>;
    fn update_movement(&mut self, movement: Movement) -> Result<bool, Self::Error>;
    fn save_subscription(&mut self, subscription: Subscription) -> Result<(), Self::Error>;
    fn list_subscriptions(&self) -> Result<Vec<Subscription>, Self::Error>;
    fn save_category_budget(&mut self, budget: CategoryBudget) -> Result<(), Self::Error>;
    fn list_category_budgets(&self) -> Result<Vec<CategoryBudget>, Self::Error>;
    fn save_savings_goal(&mut self, goal: SavingsGoal) -> Result<(), Self::Error>;
    fn list_savings_goals(&self) -> Result<Vec<SavingsGoal>, Self::Error>;
    fn save_portfolio_holding(&mut self, holding: PortfolioHolding) -> Result<(), Self::Error>;
    fn list_portfolio_holdings(&self) -> Result<Vec<PortfolioHolding>, Self::Error>;
    fn save_bank_offer(&mut self, offer: BankOffer) -> Result<(), Self::Error>;
    fn list_bank_offers(&self) -> Result<Vec<BankOffer>, Self::Error>;
}

pub fn record_movement<R: LedgerRepository>(
    repository: &mut R,
    movement: Movement,
) -> Result<(), R::Error> {
    repository.save_movement(movement)
}

pub fn record_subscription<R: LedgerRepository>(
    repository: &mut R,
    subscription: Subscription,
) -> Result<(), R::Error> {
    repository.save_subscription(subscription)
}

pub fn record_category_budget<R: LedgerRepository>(
    repository: &mut R,
    budget: CategoryBudget,
) -> Result<(), R::Error> {
    repository.save_category_budget(budget)
}

pub fn record_savings_goal<R: LedgerRepository>(
    repository: &mut R,
    goal: SavingsGoal,
) -> Result<(), R::Error> {
    repository.save_savings_goal(goal)
}

pub fn record_portfolio_holding<R: LedgerRepository>(
    repository: &mut R,
    holding: PortfolioHolding,
) -> Result<(), R::Error> {
    repository.save_portfolio_holding(holding)
}

pub fn record_bank_offer<R: LedgerRepository>(
    repository: &mut R,
    offer: BankOffer,
) -> Result<(), R::Error> {
    repository.save_bank_offer(offer)
}

/// Elimina solo el movimiento solicitado. La interfaz debe pedir confirmación antes de llamarlo.
pub fn remove_movement<R: LedgerRepository>(
    repository: &mut R,
    movement_id: &str,
) -> Result<bool, R::Error> {
    repository.delete_movement(movement_id)
}

pub fn revise_movement<R: LedgerRepository>(
    repository: &mut R,
    movement: Movement,
) -> Result<bool, R::Error> {
    repository.update_movement(movement)
}

pub fn calculate_balance(income: Money, expenses: Money) -> Option<Money> {
    BudgetSummary { income, expenses }.balance()
}

pub fn calculate_current_summary<R: LedgerRepository>(
    repository: &R,
) -> Result<Result<BudgetSummary, finanzas_domain::DomainError>, R::Error> {
    repository
        .list_movements()
        .map(|movements| BudgetSummary::from_movements(&movements))
}
