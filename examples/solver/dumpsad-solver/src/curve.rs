use cosmwasm_std::{Isqrt, Uint128};

// curve y = a*1000_000_000 - (b*1000_000_000) / (30 * 1_000_000_000 + x)
// both SOL and meme has 9 decimals
// where: y: total meme minted, x: total sol deposited

pub struct BondingCurve {
    pub a: Uint128,
    pub b: Uint128,
    pub x: Uint128,
    pub y: Uint128,
    pub graduate_threshold: Uint128,
    pub meme_cap: Uint128,
}

impl BondingCurve {
    pub const PRECISION_MULTIPLIER: Uint128 = Uint128::new(1_000_000_000u128);
    const DEFAULT_MEME_LIMIT: Uint128 = Uint128::new(1073000191 * 1_000_000_000u128);
    const DEFAULT_SOL_LIMIT: Uint128 = Uint128::new(32190005730 * 1_000_000_000u128);

    pub fn default(x: Uint128, y: Uint128, graduate_threshold: Uint128, meme_cap: Uint128) -> Self {
        BondingCurve {
            a: Self::DEFAULT_MEME_LIMIT,
            b: Self::DEFAULT_SOL_LIMIT,
            x,
            y,
            graduate_threshold,
            meme_cap,
        }
    }

    pub fn price(&self) -> Uint128 {
        // Price: (30 + x)^2 / b
        let tmp = Uint128::new(30) * BondingCurve::PRECISION_MULTIPLIER + self.x;
        (tmp * tmp) / self.b
    }

    // dY = delta Y, dX = delta X
    pub fn buy(&mut self, dx: Uint128) -> (Uint128, Uint128) {
        // y = a - b / (30 + x) (recall: y is the minted amount for user, counting from 0, not 10^9)
        // newY = a - b / (30 + newX)
        // newX = x + dx
        // newY = y + dy => dy = newY - y
        let graduate_sol_amount = (self.graduate_threshold * self.b / self.meme_cap).isqrt()
            - Uint128::new(30) * BondingCurve::PRECISION_MULTIPLIER;

        let excess_x_amount = if self.x + dx > graduate_sol_amount {
            self.x + dx - graduate_sol_amount
        } else {
            Uint128::zero()
        };

        let actual_x = dx - excess_x_amount;
        let new_x = Uint128::new(30) * BondingCurve::PRECISION_MULTIPLIER + self.x + actual_x;
        let new_y = self.a - (self.b * BondingCurve::PRECISION_MULTIPLIER) / new_x;
        let dy = new_y - self.y;

        // Update state
        self.x += actual_x;
        self.y = new_y;

        (dy, excess_x_amount)
    }

    pub fn sell(&mut self, dy: Uint128) -> Uint128 {
        // y = a - b / (30 + x) (recall: y is the minted amount for user, counting from 0, not 10^9)
        // newY = a - b / (30 + newX)
        // newY = y - dy
        // newX = x - dx => dx = x - newX
        let new_y = self.y - dy;
        let new_x = (self.b * BondingCurve::PRECISION_MULTIPLIER) / (self.a - new_y);
        let dx = Uint128::new(30) * BondingCurve::PRECISION_MULTIPLIER + self.x - new_x;
        // Update state
        self.x -= dx;
        self.y = new_y;

        dx
    }
}
