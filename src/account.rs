use crate::error::EngineErr;
use rust_decimal::Decimal;

// Represents a client's account, storing/managing balances and status
#[derive(Debug, Clone)]
pub struct Account {
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new() -> Account {
        Account {
            available: Decimal::new(0, 0),
            held: Decimal::new(0, 0),
            total: Decimal::new(0, 0),
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: Decimal) -> Result<(), EngineErr> {
        if self.locked {
            return Err(EngineErr::AccountLocked);
        }
        self.available += amount;
        self.total += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), EngineErr> {
        if self.locked {
            return Err(EngineErr::AccountLocked);
        }
        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
            Ok(())
        } else {
            Err(EngineErr::InsufficientFunds)
        }
    }

    pub fn apply_dispute(&mut self, amount: Decimal) -> Result<(), EngineErr> {
        if self.locked {
            return Err(EngineErr::AccountLocked);
        }
        if self.available >= amount {
            self.available -= amount;
            self.held += amount;
            Ok(())
        } else {
            Err(EngineErr::InsufficientFunds)
        }
    }

    pub fn resolve_dispute(&mut self, amount: Decimal) -> Result<(), EngineErr> {
        if self.locked {
            return Err(EngineErr::AccountLocked);
        }
        if self.held >= amount {
            self.held -= amount;
            self.available += amount;
            Ok(())
        } else {
            Err(EngineErr::InsufficientFunds)
        }
    }

    pub fn chargeback(&mut self, amount: Decimal) -> Result<(), EngineErr> {
        if self.locked {
            return Err(EngineErr::AccountLocked);
        }
        if self.held >= amount {
            self.total -= amount;
            self.held -= amount;
            self.locked = true;
            Ok(())
        } else {
            Err(EngineErr::InsufficientFunds)
        }
    }
}
