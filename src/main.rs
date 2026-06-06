use std::{thread, time::Duration};

use blackjack::{
    Card, GameOutcome, PlayerDrawResult, SLEEP_TIME, get_deck_shuffled, get_number_input, hand_str,
    hand_value, handle_outcome, initial_deal, is_blackjack, player_draw, print_dealer_hand,
    resolve_hand,
};

use console::Term;

fn main() {
    let term = Term::stdout();

    let mut balance: u32 = 100;
    loop {
        println!(
            "---------------- New hand (Bal: {}) ----------------",
            balance
        );
        if balance < 5 {
            println!("Balance too low. you lose");
            break;
        }

        let mut bet = u32::MAX;
        while bet > balance || bet < 5 {
            let def = balance.min(10);
            bet = get_number_input(
                &term,
                format!("Bet [min 5, max {}, def {}]", balance, def).as_str(),
                def,
            );
        }

        let mut deck = get_deck_shuffled(None);
        let mut player_hand: Vec<Card> = Vec::new();
        let mut dealer_hand: Vec<Card> = Vec::new();
        let mut outcome = None;
        let mut split: Option<(Vec<Card>, Vec<Card>)> = None;
        let mut bets = (0, 0);
        initial_deal(&mut deck, &mut player_hand, &mut dealer_hand);

        thread::sleep(Duration::from_secs_f32(SLEEP_TIME));
        println!("Dealer {}", hand_str(&dealer_hand, true));
        thread::sleep(Duration::from_secs_f32(SLEEP_TIME));
        println!(
            "You [{}] {}",
            hand_value(&player_hand).count,
            hand_str(&player_hand, false)
        );
        thread::sleep(Duration::from_secs_f32(SLEEP_TIME));

        if is_blackjack(&player_hand) || is_blackjack(&dealer_hand) {
            outcome = match (is_blackjack(&player_hand), is_blackjack(&dealer_hand)) {
                (true, true) => Some(GameOutcome::Push),
                (true, false) => Some(GameOutcome::WinByBlackjack),
                (false, true) => Some(GameOutcome::LoseByBlackjack),
                _ => panic!("Blackjack hand outcome error"),
            };
        } else {
            // Player draw
            PlayerDrawResult(split, bets) =
                player_draw(&term, &mut deck, &mut player_hand, balance, bet, true);
            if split.is_none() {
                bet = bets.0;
            }
        }
        // print_dealer_hand(&dealer_hand);

        if hand_value(&player_hand).count > 21 && outcome.is_none() {
            outcome = Some(GameOutcome::LoseByBust);
        }

        // Dealer draw
        let mut dv = hand_value(&dealer_hand);
        if outcome.is_none() {
            while (dv.count <= 16) | (dv.count <= 17 && dv.is_soft) {
                dealer_hand.push(deck.pop().unwrap());
                dv = hand_value(&dealer_hand);
                print!("[Dealer Hit] ");
                print_dealer_hand(&dealer_hand);
                thread::sleep(Duration::from_secs_f32(SLEEP_TIME));
            }

            if dv.count > 21 {
                outcome = Some(GameOutcome::WinByBust);
            } else {
                print!("[Dealer Stand] ");
                print_dealer_hand(&dealer_hand);
            }
        }
        thread::sleep(Duration::from_secs_f32(SLEEP_TIME));

        match split {
            None => {
                if outcome.is_none() {
                    outcome = Some(resolve_hand(&player_hand, &dealer_hand));
                }
                let dv = hand_value(&dealer_hand);
                let pv = hand_value(&player_hand);
                handle_outcome(outcome.unwrap(), dv, pv, bet, &mut balance);
            }
            Some(split) => {
                let dv = hand_value(&dealer_hand);
                let pv = hand_value(&split.0);
                let split_outcome = outcome.unwrap_or(resolve_hand(&split.0, &dealer_hand));
                handle_outcome(split_outcome, dv, pv, bets.0, &mut balance);
                thread::sleep(Duration::from_secs_f32(SLEEP_TIME));
                let pv = hand_value(&split.1);
                let split_outcome = outcome.unwrap_or(resolve_hand(&split.0, &dealer_hand));
                handle_outcome(split_outcome, dv, pv, bets.1, &mut balance);
            }
        }
        thread::sleep(Duration::from_secs_f32(SLEEP_TIME));
    }
}
