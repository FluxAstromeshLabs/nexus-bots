use cosmwasm_std::Uint128;

// curve y = a - b / (30 + x)
// where: y: total meme minted, x: total sol deposited
pub struct BondingCurve {
    pub a: Uint128,
    pub b: Uint128,
    pub x: Uint128,
    pub y: Uint128,
}

impl BondingCurve {
    const DEFAULT_A: Uint128 = Uint128::new(1073000191);
    const DEFAULT_B: Uint128 = Uint128::new(32190005730);
    const SOL_BPS: Uint128 = Uint128::new(1_000_000_000);
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
        let numerator = (Uint128::new(30) + self.x) * (Uint128::new(30) + self.x) * BondingCurve::SOL_BPS;
        numerator / self.b
    }

    // dY = delta Y, dX = delta X
    pub fn buy(&mut self, dx: Uint128) -> Uint128 {
        // dY = y - a + b / (30 + x + dX)
        let new_x = Uint128::new(30) + self.x + dx;
        let new_y = self.a - self.b / new_x;
        let dy = new_y - self.y;

        // Update state
        self.x += dx;
        self.y = new_y;

        dy
    }

    pub fn sell(&mut self, dy: Uint128) -> Uint128 {
        // dX = 30 + x - b / (y - dY - a)
        let new_y = self.y - dy;
        let new_x = Uint128::new(30) + self.x - self.b / (new_y - self.a);
        let dx = self.x - new_x;

        // Update state
        self.x = new_x;
        self.y = new_y;

        dx
    }
}
