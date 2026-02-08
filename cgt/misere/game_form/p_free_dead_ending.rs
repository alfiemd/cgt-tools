#![allow(missing_docs)]

use crate::{
    misere::game_form::{DeadEndingContext, PFreeContext},
    short::partizan::Player,
};

pub trait PFreeDeadEndingContext: DeadEndingContext + PFreeContext {
    fn ge_mod_p_free_dead_ending(&self, g: &Self::Form, h: &Self::Form) -> bool {
        if let Some(g) = self.to_integer(g)
            && let Some(h) = self.to_integer(h)
        {
            return g <= h;
        }
        self.ge_mod_dead_ending(g, h)
    }

    fn eq_mod_p_free_dead_ending(&self, g: &Self::Form, h: &Self::Form) -> bool {
        self.ge_mod_p_free_dead_ending(g, h) && self.ge_mod_p_free_dead_ending(h, g)
    }

    fn eliminate_dominated_moves(&self, moves: &mut Vec<Self::Form>, player: Player) {
        let mut i = 0;
        'loop_i: while i < moves.len() {
            let mut j = i + 1;
            'loop_j: while i < moves.len() && j < moves.len() {
                let move_i = &moves[i];
                let move_j = &moves[j];

                let remove_i = match player {
                    Player::Left => self.ge_mod_p_free_dead_ending(move_j, move_i),
                    Player::Right => self.ge_mod_p_free_dead_ending(move_i, move_j),
                };

                if remove_i {
                    moves.swap_remove(i);
                    continue 'loop_i;
                }

                let remove_j = match player {
                    Player::Left => self.ge_mod_p_free_dead_ending(move_i, move_j),
                    Player::Right => self.ge_mod_p_free_dead_ending(move_j, move_i),
                };

                if remove_j {
                    moves.swap_remove(j);
                    continue 'loop_j;
                }

                j += 1;
            }

            i += 1;
        }
    }

    fn reduced(&self, game: &Self::Form) -> Self::Form {
        let mut left = self.moves(game, Player::Left).cloned().collect::<Vec<_>>();
        self.eliminate_dominated_moves(&mut left, Player::Left);

        let mut right = self.moves(game, Player::Right).cloned().collect::<Vec<_>>();
        self.eliminate_dominated_moves(&mut right, Player::Right);

        if let [gl] = left.as_slice()
            && let Some(a) = self.to_integer(gl)
            && let [gr] = right.as_slice()
            && let Some(b) = self.to_integer(gr)
        {
            // TODO: Is there a general rule for this?
            if a == -1 && b == 1 {
                return self.new_integer(0).unwrap();
            }

            if a >= 0 && b <= a + 2 {
                return self.new_integer(a + 1).unwrap();
            }

            if b <= 0 && a >= b - 2 {
                return self.new_integer(b - 1).unwrap();
            }
        }

        self.new(left, right).unwrap()
    }
}

impl<C> PFreeDeadEndingContext for C where C: DeadEndingContext + PFreeContext {}

#[cfg(test)]
mod tests {
    use crate::{
        misere::game_form::{
            DeadEndingContext, DeadEndingFormContext, GameFormContext, PFreeDeadEndingContext,
            PFreeFormContext, StandardFormContext,
        },
        total::TotalWrappable,
    };

    #[test]
    fn relations() {
        let context = PFreeFormContext::new(DeadEndingFormContext::new(StandardFormContext));

        let g = context.from_str("0").unwrap();
        let h = context.from_str("1").unwrap();
        assert!(context.ge_mod_p_free_dead_ending(&g, &h));
        assert!(!context.ge_mod_dead_ending(&g, &h));

        let g = context.from_str("{1|3}").unwrap();
        let h = context.from_str("2").unwrap();
        assert!(context.eq_mod_p_free_dead_ending(&g, &h));
    }

    #[test]
    fn reductions() {
        let context = PFreeFormContext::new(DeadEndingFormContext::new(StandardFormContext));

        let g = context.reduced(&context.from_str("{0,1|3}").unwrap());
        let h = context.from_str("{0|3}").unwrap();
        assert!(TotalWrappable::total_eq(&g, &h));

        let g = context.reduced(&context.from_str("{0|2}").unwrap());
        let h = context.from_str("1").unwrap();
        assert!(TotalWrappable::total_eq(&g, &h));

        let g = context.reduced(&context.from_str("{-2|0}").unwrap());
        let h = context.from_str("-1").unwrap();
        assert!(TotalWrappable::total_eq(&g, &h));
    }
}
