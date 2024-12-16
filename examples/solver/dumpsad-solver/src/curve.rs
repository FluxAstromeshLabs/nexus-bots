use cosmwasm_std::Uint128;

// curve y = a*1000_000_000 - (b*1000_000_000) / (30 * 1_000_000_000 + x)
// both SOL and meme has 9 decimals
// where: y: total meme minted, x: total sol deposited
pub struct BondingCurve {
    pub a: Uint128,
    pub b: Uint128,
    pub x: Uint128,
    pub y: Uint128,
}

// 67062511 = 1073000191-32190005730/(32)
// 32190005730/(1073000191 - 50000000)

impl BondingCurve {
    const PRECISION_MULTIPLIER: Uint128 = Uint128::new(1_000_000_000u128);
    const DEFAULT_A: Uint128 = Uint128::new(1073000191 * 1_000_000_000u128);
    const DEFAULT_B: Uint128 = Uint128::new(32190005730 * 1_000_000_000u128);

    pub fn default(x: Uint128, y: Uint128) -> Self {
        BondingCurve {
            a: Self::DEFAULT_A,
            b: Self::DEFAULT_B,
            x,
            y,
        }
    }

    pub fn price(&self) -> Uint128 {
        // Price: (30 + x)^2 / b
        let tmp = Uint128::new(30) * BondingCurve::PRECISION_MULTIPLIER + self.x;
        (tmp * tmp) / self.b
    }

    // dY = delta Y, dX = delta X
    pub fn buy(&mut self, dx: Uint128) -> Uint128 {
        // y = a - b / (30 + x) (recall: y is the minted amount for user, counting from 0, not 10^9)
        // newY = a - b / (30 + newX)
        // newX = x + dx
        // newY = y + dy => dy = newY - y
        let new_x = Uint128::new(30) * BondingCurve::PRECISION_MULTIPLIER + self.x + dx;
        let new_y = self.a - (self.b * BondingCurve::PRECISION_MULTIPLIER) / new_x;
        let dy = new_y - self.y;

        // Update state
        self.x += dx;
        self.y = new_y;

        dy
    }

    pub fn sell(&mut self, dy: Uint128) -> Uint128 {
        // y = a - b / (30 + x) (recall: y is the minted amount for user, counting from 0, not 10^9)
        // newY = a - b / (30 + newX)
        // newY = y - dy
        // newX = x - dx => dx = x - newX
        let new_y = self.y - dy; // (10^9*10^9 - 932937488062500000 - 10000000000000) = 67052511937500000
        let new_x = (self.b * BondingCurve::PRECISION_MULTIPLIER) / (self.a - new_y); // 32190005730*10^9*10^9 / (1073000191*10^9 - 67052511937500000)
                                                                                      // new_x = 30+x-dx => dx = 30+x-new_x
        let dx = Uint128::new(30) * BondingCurve::PRECISION_MULTIPLIER + self.x - new_x;
        // Update state
        self.x = new_x;
        self.y = new_y;

        dx
    }
}
