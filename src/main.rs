use std::fmt::Display;

use indexmap::IndexMap;
use itertools::Itertools;
use rand::{rngs::ThreadRng, seq::SliceRandom, Rng};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Debug, Clone)]
pub struct Dice {
    value: u8,
    rng: ThreadRng,
}

impl Dice {
    pub fn new() -> Self {
        Self::new_with_value(1)
    }

    pub fn new_with_value(value: u8) -> Self {
        let rng = rand::thread_rng();
        Self { value, rng }
    }

    pub fn roll(&mut self) {
        self.value = self.rng.gen_range(1..=6);
    }
}

impl Default for Dice {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Pool {
    dice: Vec<Dice>,
    pub map: IndexMap<u8, usize>,
}

impl Pool {
    pub fn new() -> Self {
        let mut map = IndexMap::new();
        map.insert(1, 8);
        Self {
            dice: vec![Dice::default(); 8],
            map,
        }
    }

    pub fn from_map(map: &IndexMap<u8, usize>) -> Self {
        let mut dice = vec![];
        for (dice_value, count) in map {
            for _ in 0..*count {
                dice.push(Dice::new_with_value(*dice_value));
            }
        }
        Self {
            dice,
            map: map.clone(),
        }
    }

    pub fn roll_pool(&mut self) {
        self.dice.iter_mut().for_each(|d| d.roll());
        self.map.clear();
        self.dice.iter().for_each(|d| {
            let count = self.map.entry(d.value).or_insert(0);
            *count += 1;
        })
    }

    pub fn reroll(&mut self, n: u8) {
        // choose n dice to reroll
        let mut range: Vec<_> = (0..self.dice.len()).collect();
        let mut rng = rand::thread_rng();
        range.shuffle(&mut rng);
        let indexes = &range[0..n as usize];
        for index in indexes {
            self.dice[*index].roll();
        }
        self.map.clear();
        self.dice.iter().for_each(|d| {
            let count = self.map.entry(d.value).or_insert(0);
            *count += 1;
        })
    }

    pub fn check_goals(&self, goals: &[Rule], rerolls: u8) -> bool {
        let mut mod_map = self.map.clone();
        for goal in goals {
            let success = Self::check_goals_inner(&mut mod_map, *goal);
            if !success && rerolls > 0 {
                let mut tmp_pool = Pool::from_map(&mod_map);
                tmp_pool.reroll(rerolls);
                let mut tmp_mod_map = tmp_pool.map.clone();
                if !Self::check_goals_inner(&mut tmp_mod_map, *goal) {
                    return false;
                } else {
                    std::mem::swap(&mut tmp_mod_map, &mut mod_map);
                    continue;
                }
            } else if !success {
                return false;
            }
        }
        true
    }

    fn check_goals_inner(mod_map: &mut IndexMap<u8, usize>, goal: Rule) -> bool {
        if let Some(to_remove) = goal.matches(&mod_map) {
            to_remove.into_iter().for_each(|r| {
                let count = mod_map.get_mut(&r.dice_value).unwrap();
                let c = *count;
                *count = c.saturating_sub(1);
            });
            true
        } else {
            // Rule was not matched
            false
        }
    }
}

impl Default for Pool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Rule {
    RageFuel,
    WrathfulDevotion,
    MartialExcellence,
    TotalCarnage,
    WarpBlades,
    UnbridledBloodlust,
    AngryRon,
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rule::RageFuel => write!(f, "+2 inch move"),
            Rule::WrathfulDevotion => write!(f, "6+ FNP"),
            Rule::MartialExcellence => write!(f, "Sustained Hits 1"),
            Rule::TotalCarnage => write!(f, "4+ Fight on Death"),
            Rule::WarpBlades => write!(f, "Lethal Hits"),
            Rule::UnbridledBloodlust => write!(f, "Advance and Charge"),
            Rule::AngryRon => write!(f, "Res Angron"),
        }
    }
}

pub struct ToRemove {
    dice_value: u8,
}

impl ToRemove {
    pub fn new(dice_value: u8) -> Self {
        Self { dice_value }
    }
}

impl Rule {
    pub fn matches(&self, pool: &IndexMap<u8, usize>) -> Option<Vec<ToRemove>> {
        match self {
            Rule::RageFuel => {
                // Any double
                let mut ret = None;
                for (dice_value, quantity) in pool {
                    if *quantity == 2 {
                        ret.replace(vec![ToRemove::new(*dice_value), ToRemove::new(*dice_value)]);
                        break;
                    }
                }
                ret
            }
            Rule::WrathfulDevotion => {
                // Any double
                let mut ret = None;
                for (dice_value, quantity) in pool {
                    if *quantity >= 2 {
                        ret.replace(vec![ToRemove::new(*dice_value), ToRemove::new(*dice_value)]);
                        break;
                    }
                }
                ret
            }
            Rule::MartialExcellence => {
                // Double 3+
                let mut ret = None;
                for (dice_value, quantity) in pool {
                    if *dice_value >= 3 && *quantity >= 2 {
                        ret.replace(vec![ToRemove::new(*dice_value), ToRemove::new(*dice_value)]);
                        break;
                    }
                }
                ret
            }
            Rule::TotalCarnage => {
                // Double 4+ or any Triple
                let mut ret = None;
                for (dice_value, quantity) in pool {
                    if *dice_value >= 4 && *quantity >= 2 {
                        ret.replace(vec![ToRemove::new(*dice_value), ToRemove::new(*dice_value)]);
                        break;
                    } else if *quantity >= 3 {
                        ret.replace(vec![
                            ToRemove::new(*dice_value),
                            ToRemove::new(*dice_value),
                            ToRemove::new(*dice_value),
                        ]);
                        break;
                    }
                }
                ret
            }
            Rule::WarpBlades => {
                // Double 5+ or any Triple
                let mut ret = None;
                for (dice_value, quantity) in pool {
                    if *dice_value >= 5 && *quantity >= 2 {
                        ret.replace(vec![ToRemove::new(*dice_value), ToRemove::new(*dice_value)]);
                        break;
                    } else if *quantity >= 3 {
                        ret.replace(vec![
                            ToRemove::new(*dice_value),
                            ToRemove::new(*dice_value),
                            ToRemove::new(*dice_value),
                        ]);
                        break;
                    }
                }
                ret
            }
            Rule::UnbridledBloodlust => {
                // Double 6 or any Triple
                let mut ret = None;
                for (dice_value, quantity) in pool {
                    if *dice_value == 6 && *quantity >= 2 {
                        ret.replace(vec![ToRemove::new(*dice_value), ToRemove::new(*dice_value)]);
                        break;
                    } else if *quantity >= 3 {
                        ret.replace(vec![
                            ToRemove::new(*dice_value),
                            ToRemove::new(*dice_value),
                            ToRemove::new(*dice_value),
                        ]);
                        break;
                    }
                }
                ret
            }
            Rule::AngryRon => {
                // Triple 6
                let mut ret = None;
                for (dice_value, quantity) in pool {
                    if *dice_value == 6 && *quantity >= 3 {
                        ret.replace(vec![ToRemove::new(*dice_value), ToRemove::new(*dice_value)]);
                        break;
                    }
                }
                ret
            }
        }
    }
}

fn main() {
    // Find the probabilities of each combination of effects when rolling 8 dice 1000 times
    for goals in Rule::iter().combinations(2) {
        for reroll in 0..=3 {
            for goals in [&[goals[0], goals[1]], &[goals[1], goals[0]]] {
                let mut pool = Pool::default();
                // let goals = [Rule::RageFuel, Rule::WrathfulDevotion];
                let iterations = 10_000;
                let mut success = 0;
                for _ in 0..iterations {
                    pool.roll_pool();
                    if pool.check_goals(goals, reroll) {
                        success += 1;
                    }
                }
                println!(
                    "({:?}({}) & {:?}({})): {}/{} or {}% of the time with {} rerolls",
                    &goals[0],
                    &goals[0],
                    &goals[1],
                    &goals[1],
                    success,
                    iterations,
                    (success as f64 / iterations as f64) * 100.0,
                    reroll
                );
            }
        }
    }
}
