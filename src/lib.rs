use std::{
    io::{self, Write},
    ops::{Add, Sub},
    thread,
    time::Duration,
};

use console::{Key, Term};
use rand::{rng, seq::SliceRandom};

pub const SLEEP_TIME: f32 = 0.25;

#[derive(Debug, Clone)]
pub enum Suit {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CardValue {
    Number(u8),
    Ace,
    Jack,
    Queen,
    King,
}

#[derive(Debug, Clone)]
pub struct Card {
    pub value: CardValue,
    pub suit: Suit,
}

#[derive(Debug, Clone, Copy)]
pub struct HandValue {
    pub count: u8,
    pub is_soft: bool,
}

#[derive(Debug)]
pub enum BlackjackAction {
    Stand,
    Hit,
    DoubleDown,
    Split,
}

pub enum EndReason {}

#[derive(Debug, Clone, Copy)]
pub enum GameOutcome {
    WinByCount,
    WinByBust,
    WinByBlackjack,
    Push,
    LoseByCount,
    LoseByBust,
    LoseByBlackjack,
}

pub fn create_deck() -> Vec<Card> {
    const SUITS: [Suit; 4] = [Suit::Hearts, Suit::Diamonds, Suit::Spades, Suit::Clubs];
    let mut cards = Vec::with_capacity(52);
    for suit in &SUITS {
        for value in 1..=10 {
            cards.push(Card {
                value: CardValue::Number(value),
                suit: suit.clone(),
            });
        }
        cards.push(Card {
            value: CardValue::Ace,
            suit: suit.clone(),
        });
        cards.push(Card {
            value: CardValue::Jack,
            suit: suit.clone(),
        });
        cards.push(Card {
            value: CardValue::Queen,
            suit: suit.clone(),
        });
        cards.push(Card {
            value: CardValue::King,
            suit: suit.clone(),
        });
    }
    cards
}

pub fn get_deck_shuffled(deck: Option<Vec<Card>>) -> Vec<Card> {
    let mut deck = deck.map_or(create_deck(), |v| v);
    let mut rng = rng();
    deck.shuffle(&mut rng);
    deck
}

pub fn initial_deal(
    deck: &mut Vec<Card>,
    player_hand: &mut Vec<Card>,
    dealer_hand: &mut Vec<Card>,
) {
    for _ in 0..2 {
        if deck.is_empty() {
            *deck = get_deck_shuffled(None);
        }
        player_hand.push(deck.pop().unwrap());
    }
    for _ in 0..2 {
        if deck.is_empty() {
            *deck = get_deck_shuffled(None);
        }
        dealer_hand.push(deck.pop().unwrap());
    }
}

pub fn get_bool_input(term: &Term, prompt: &str) -> bool {
    print!("{prompt} (y/n)");
    let _ = io::stdout().flush();
    while let Ok(key) = term.read_char() {
        match key.to_ascii_lowercase() {
            'y' => {
                println!();
                return true;
            }
            'n' => {
                println!();
                return false;
            }
            _ => {
                let _ = term.clear_line();
                print!("{prompt} (y/n only)> ");
                let _ = io::stdout().flush();
            }
        }
    }
    false
}

pub fn get_number_input(term: &Term, prompt: &str, default: u32) -> u32 {
    let mut input_str = String::new();
    print!("{prompt}> ");
    let _ = io::stdout().flush();
    while let Ok(key) = term.read_key() {
        match key {
            Key::Char(key) => match key {
                key if key.is_ascii_digit() => {
                    input_str.push(key);
                    print!("{key}");
                    let _ = io::stdout().flush();
                    continue;
                }
                _ => (),
            },
            Key::Backspace => {
                let _ = input_str.pop();
                let _ = term.clear_line();
                print!("{prompt}> {}", input_str);
                let _ = io::stdout().flush();
                continue;
            }
            Key::Enter if input_str.is_empty() => break,
            Key::Enter => match input_str.parse::<u32>() {
                Ok(n) => {
                    println!();
                    return n;
                }
                Err(_) => {
                    let _ = term.clear_line();
                    print!("{prompt} (invalid input)> ");
                    let _ = io::stdout().flush();
                    continue;
                }
            },

            _ => (),
        }
    }
    println!("{default}");
    default
}

pub fn get_action_input(
    term: &Term,
    can_split: bool,
    can_double_down: bool,
) -> io::Result<BlackjackAction> {
    let prompt = {
        let mut default = String::from("(H)it, (S)tand");
        if can_split {
            default.push_str(", (X) Split");
        }
        if can_double_down {
            default.push_str(", (D)ouble Down");
        }
        default
    };
    print!("{}: ", prompt);
    let _ = io::stdout().flush();
    loop {
        let key = term.read_char()?;
        match key.to_ascii_lowercase() {
            'h' => {
                let _ = term.clear_line();
                print!("[You Hit] ");
                return Ok(BlackjackAction::Hit);
            }
            's' => {
                let _ = term.clear_line();
                print!("[You Stand] ");
                return Ok(BlackjackAction::Stand);
            }
            'x' if can_split => {
                let _ = term.clear_line();
                print!("[You Split] ");
                return Ok(BlackjackAction::Split);
            }
            'd' if can_double_down => {
                let _ = term.clear_line();
                print!("[You Double Down] ");
                return Ok(BlackjackAction::DoubleDown);
            }
            _ => (),
        }
    }
}

pub fn card_str(card: &Card) -> String {
    let mut card_str = String::new();

    card_str.push_str(&match card.value {
        CardValue::Number(v) => v.to_string(),
        CardValue::Ace => "A".to_string(),
        CardValue::Jack => "J".to_string(),
        CardValue::Queen => "Q".to_string(),
        CardValue::King => "K".to_string(),
    });

    card_str.push(match card.suit {
        Suit::Hearts => '♡',
        Suit::Diamonds => '♢',
        Suit::Spades => '♤',
        Suit::Clubs => '♧',
    });

    card_str
}

pub fn hand_str(hand: &[Card], only_show_first: bool) -> String {
    let mut hand_str = String::new();

    if hand.len() < 2 {
        return hand_str;
    }

    if only_show_first {
        hand_str.push_str(&card_str(hand.first().unwrap()));
        hand_str.push(' ');
        for _ in 0..hand.len() - 1 {
            hand_str.push_str("▒▒ ");
        }
    } else {
        hand.iter().for_each(|c| {
            hand_str.push_str(&card_str(c));
            hand_str.push(' ');
        });
    }

    hand_str
}

pub fn print_player_hand(hand: &[Card]) {
    let pv = hand_value(hand);
    println!(
        "Your hand: [{}{}] {}",
        if pv.is_soft { "*" } else { "" },
        pv.count,
        hand_str(hand, false)
    );
}

pub fn print_dealer_hand(hand: &[Card]) {
    let dv = hand_value(hand);
    println!(
        "Dealer hand: [{}{}] {}",
        if dv.is_soft { "*" } else { "" },
        dv.count,
        hand_str(hand, false)
    );
}

pub fn hand_value(hand: &[Card]) -> HandValue {
    let mut ace_count = hand
        .iter()
        .filter(|c| matches!(c.value, CardValue::Ace))
        .count();
    let mut hand_value: u16 = hand
        .iter()
        .map(|c| match c.value {
            CardValue::Number(v) => v as u16,
            CardValue::Ace => 11,
            CardValue::Jack => 10,
            CardValue::Queen => 10,
            CardValue::King => 10,
        })
        .sum();

    while ace_count > 0 && hand_value > 21 {
        hand_value -= 10;
        ace_count -= 1;
    }

    HandValue {
        count: hand_value as u8,
        is_soft: ace_count > 0,
    }
}

pub fn is_blackjack(hand: &[Card]) -> bool {
    hand.len() == 2 && hand_value(hand).count == 21
}

pub struct PlayerDrawResult(pub Option<(Vec<Card>, Vec<Card>)>, pub (u32, u32));

pub fn player_draw(
    term: &Term,
    deck: &mut Vec<Card>,
    player_hand: &mut Vec<Card>,
    balance: u32,
    bet: u32,
    is_split: bool,
) -> PlayerDrawResult {
    let mut hit_count = 0;
    let mut bet = bet;
    'action: while hand_value(player_hand).count < 21 {
        fn card_split_value(card: &CardValue) -> u8 {
            match card {
                CardValue::Number(v) => *v,
                CardValue::Ace => 11,
                CardValue::Jack | CardValue::Queen | CardValue::King => 10,
            }
        }
        let can_afford = hit_count == 0 && bet * 2 <= balance;
        let cards_similar = can_afford
            && card_split_value(&player_hand[0].value) == card_split_value(&player_hand[1].value);

        match get_action_input(term, cards_similar && !is_split, can_afford) {
            Ok(a) => match a {
                BlackjackAction::Stand => {
                    print_player_hand(player_hand);
                    thread::sleep(Duration::from_secs_f32(SLEEP_TIME));
                    break 'action;
                }
                BlackjackAction::Hit => {
                    hit_count += 1;
                    player_hand.push(deck.pop().unwrap());
                    print_player_hand(player_hand);
                    thread::sleep(Duration::from_secs_f32(SLEEP_TIME));
                }
                BlackjackAction::DoubleDown => {
                    bet *= 2;
                    player_hand.push(deck.pop().unwrap());
                    print_player_hand(player_hand);
                    thread::sleep(Duration::from_secs_f32(SLEEP_TIME));
                    break 'action;
                }
                BlackjackAction::Split => {
                    let first_card = player_hand.first().unwrap().clone();
                    let first_ace = matches!(first_card.value, CardValue::Ace);
                    let mut hand1 = vec![first_card];
                    hand1.push(deck.pop().unwrap());

                    let second_card = player_hand.get(1).unwrap().clone();
                    let second_ace = matches!(second_card.value, CardValue::Ace);
                    let mut hand2 = vec![second_card];
                    hand2.push(deck.pop().unwrap());

                    println!(
                        "Your new hands:\n{}\n{}",
                        hand_str(&hand1, false),
                        hand_str(&hand2, false),
                    );

                    let betinfirst = if !first_ace {
                        println!("First hand:");
                        print_player_hand(&hand1);
                        let PlayerDrawResult(_, (b, _)) =
                            player_draw(term, deck, &mut hand1, balance, bet, false);
                        b
                    } else {
                        bet
                    };
                    let betinsecond = if !second_ace {
                        println!("Second hand:");
                        print_player_hand(&hand2);
                        let PlayerDrawResult(_, (_, b)) =
                            player_draw(term, deck, &mut hand2, balance - betinfirst, bet, false);
                        b
                    } else {
                        bet
                    };

                    return PlayerDrawResult(Some((hand1, hand2)), (betinfirst, betinsecond));
                }
            },
            Err(e) => panic!("{}", e),
        }
    }
    PlayerDrawResult(None, (bet, 0))
}

pub fn resolve_hand(player_hand: &[Card], dealer_hand: &[Card]) -> GameOutcome {
    let dv = hand_value(dealer_hand);
    let pv = hand_value(player_hand);
    match pv.count.cmp(&dv.count) {
        std::cmp::Ordering::Less => GameOutcome::LoseByCount,
        std::cmp::Ordering::Equal => GameOutcome::Push,
        std::cmp::Ordering::Greater => GameOutcome::WinByCount,
    }
}

pub fn handle_outcome(
    outcome: GameOutcome,
    dv: HandValue,
    pv: HandValue,
    bet: u32,
    balance: &mut u32,
) {
    match outcome {
        GameOutcome::WinByCount => {
            println!("You win! ({} to {}) +{}", pv.count, dv.count, bet);
        }
        GameOutcome::WinByBust => {
            println!("You win! Dealer busted with {}. +{}", dv.count, bet)
        }
        GameOutcome::WinByBlackjack => {
            let bet = (bet as f64 * 1.5) as u32;
            println!("Blackjack! +{}", bet)
        }
        GameOutcome::Push => {
            println!("Push. ({} to {})", pv.count, dv.count)
        }
        GameOutcome::LoseByCount => {
            println!("You lose. ({} to {}) -{}", dv.count, pv.count, bet);
        }
        GameOutcome::LoseByBust => {
            println!("You lose. You busted with {}. -{}", pv.count, bet)
        }
        GameOutcome::LoseByBlackjack => {
            println!("Dealer blackjack! -{}", bet);
        }
    }
    *balance = match outcome {
        GameOutcome::WinByCount | GameOutcome::WinByBust | GameOutcome::WinByBlackjack => {
            balance.add(bet)
        }
        GameOutcome::Push => *balance,
        GameOutcome::LoseByCount | GameOutcome::LoseByBust | GameOutcome::LoseByBlackjack => {
            balance.sub(bet)
        }
    }
}
