use crate::position::GameResult;
use crate::position::GameResult::*;
use std::fmt;
use std::fmt::Formatter;

/// Sum of all six fields will always be 1.0
#[derive(Clone, Default, PartialEq)]
pub struct Probabilities {
    pub win_normal: f32,
    pub win_gammon: f32,
    pub lose_normal: f32,
    pub lose_gammon: f32,
}

impl fmt::Debug for Probabilities {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Probabilities: wn {:.2}%; wg {:.2}%; ln {:.2}%; lg {:.2}%",
            100.0 * self.win_normal,
            100.0 * self.win_gammon,
            100.0 * self.lose_normal,
            100.0 * self.lose_gammon,
        )
    }
}

/// Used when writing CSV data to a file
impl fmt::Display for Probabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{};{};{};{}",
            self.win_normal,
            self.win_gammon,
            self.lose_normal,
            self.lose_gammon,
        )
    }
}

impl Probabilities {
    pub fn csv_header() -> String {
        "win_normal;win_gammon;lose_normal;lose_gammon".to_string()
    }

    pub fn win(&self) -> f32 {
        self.win_normal + self.win_gammon
    }

    pub(crate) fn switch_sides(&self) -> Self {
        Self {
            win_normal: self.lose_normal,
            win_gammon: self.lose_gammon,
            lose_normal: self.win_normal,
            lose_gammon: self.win_gammon,
        }
    }

    /// Cubeless equity
    pub fn equity(&self) -> f32 {
        self.win_normal - self.lose_normal
            + 2.0 * (self.win_gammon - self.lose_gammon)
    }
}

impl From<&ResultCounter> for Probabilities {
    /// Typically used from rollouts.
    fn from(value: &ResultCounter) -> Self {
        let sum = value.sum() as f32;
        Probabilities {
            win_normal: value.num_of(WinNormal) as f32 / sum,
            win_gammon: value.num_of(WinGammon) as f32 / sum,
            lose_normal: value.num_of(LoseNormal) as f32 / sum,
            lose_gammon: value.num_of(LoseGammon) as f32 / sum,
        }
    }
}

#[derive(Default)]
pub struct ResultCounter {
    results: [u32; 4],
}

impl ResultCounter {
    /// Convenience method, mainly for tests
    pub fn new(
        win_normal: u32,
        win_gammon: u32,
        lose_normal: u32,
        lose_gammon: u32,
    ) -> Self {
        let results = [
            win_normal,
            win_gammon,
            lose_normal,
            lose_gammon,
        ];
        Self { results }
    }
    pub fn add(&mut self, result: GameResult) {
        self.results[result as usize] += 1;
    }

    pub fn add_results(&mut self, result: GameResult, amount: u32) {
        self.results[result as usize] += amount;
    }

    pub fn sum(&self) -> u32 {
        self.results.iter().sum::<u32>()
    }

    pub fn num_of(&self, result: GameResult) -> u32 {
        // This works because the enum has associated integer values (discriminant), starting with zero.
        self.results[result as usize]
    }

    pub fn combine(self, counter: &ResultCounter) -> Self {
        let mut results = self.results;
        for (self_value, counter_value) in results.iter_mut().zip(counter.results) {
            *self_value += counter_value;
        }
        Self { results }
    }
}

#[cfg(test)]
mod tests {
    use crate::position::GameResult::{LoseGammon, LoseNormal, WinGammon};
    use crate::probabilities::{Probabilities, ResultCounter};

    #[test]
    fn from_result_counter() {
        // sum of `results is 32, a power of 2. Makes fractions easier to handle.
        let mut counter = ResultCounter::default();
        counter.add(WinGammon);
        counter.add_results(LoseNormal, 4);
        counter.add_results(LoseGammon, 8);

        let probabilities = Probabilities::from(&counter);
        assert_eq!(probabilities.win_normal, 0.0);
        assert_eq!(probabilities.win_gammon, 0.03125);
        assert_eq!(probabilities.lose_normal, 0.125);
        assert_eq!(probabilities.lose_gammon, 0.25);
    }

    #[test]
    fn to_string() {
        let probabilities = Probabilities {
            win_normal: 1.0 / 21.0,
            win_gammon: 2.0 / 21.0,
            lose_normal: 4.0 / 21.0,
            lose_gammon: 5.0 / 21.0,
        };
        assert_eq!(
            probabilities.to_string(),
            "0.04761905;0.0952381;0.14285715;0.1904762;0.23809524;0.2857143"
        );
    }

    #[test]
    fn equity_win_normal() {
        let probabilities = Probabilities {
            win_normal: 1.0,
            win_gammon: 0.0,
            lose_normal: 0.0,
            lose_gammon: 0.0,
        };
        assert_eq!(probabilities.equity(), 1.0);
    }

    #[test]
    fn equity_win_gammon() {
        let probabilities = Probabilities {
            win_normal: 0.0,
            win_gammon: 1.0,
            lose_normal: 0.0,
            lose_gammon: 0.0,
        };
        assert_eq!(probabilities.equity(), 2.0);
    }

    #[test]
    fn equity_lose_normal() {
        let probabilities = Probabilities {
            win_normal: 0.0,
            win_gammon: 0.0,
            lose_normal: 1.0,
            lose_gammon: 0.0,
        };
        assert_eq!(probabilities.equity(), -1.0);
    }

    #[test]
    fn equity_lose_gammon() {
        let probabilities = Probabilities {
            win_normal: 0.0,
            win_gammon: 0.0,
            lose_normal: 0.0,
            lose_gammon: 1.0,
        };
        assert_eq!(probabilities.equity(), -2.0);
    }

    #[test]
    fn equity_balanced() {
        let probabilities = Probabilities {
            win_normal: 0.3,
            win_gammon: 0.1,
            lose_normal: 0.3,
            lose_gammon: 0.1,
        };
        assert_eq!(probabilities.equity(), 0.0);
    }

    #[test]
    fn win() {
        let probabilities = Probabilities {
            win_normal: 0.5,
            win_gammon: 0.2,
            lose_normal: 0.1,
            lose_gammon: 0.07,
        };
        assert_eq!(probabilities.win(), 0.82);
    }
}
