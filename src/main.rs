use eframe::egui;
use rand::{Rng};
//use serde::de;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Card {
    rank: u8,
    suit: Suit,
}
impl Card {
    fn value(&self) -> u8 {
        match self.rank {
            1 => 11, // Ace
            11 | 12 | 13 => 10, // Face cards
            _ => self.rank,
        }
    }

    fn name(&self) -> String {
        let rank_str = match self.rank {
            1 => "A".to_string(),
            11 => "J".to_string(),
            12 => "Q".to_string(),
            13 => "K".to_string(),
            _ => self.rank.to_string(),
        };
        let suit_str = match self.suit {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        };
        format!("{}{}", rank_str, suit_str)
    }
}

#[derive(Debug, Clone)]
struct Hand {
    cards: Vec<Card>,
}
impl Hand {
    fn new() -> Self {
        Self { cards: Vec::new() }
    }

    fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    fn total(&self) -> u8 {
        let mut total = 0;
        let mut aces = 0;
        for card in &self.cards {
            total += card.value();
            if card.rank == 1 {
                aces += 1;
            }
        }
        while total > 21 && aces > 0 {
            total -= 10;
            aces -= 1;
        }
        total
    }

    fn is_blackjack(&self) -> bool {
        self.cards.len() == 2 && self.total() == 21
    }

    fn is_busted(&self) -> bool {
        self.total() > 21
    }

    fn display(&self) -> String {
        self.cards.iter()
            .map(|c| c.name())
            .collect::<Vec<_>>()
            .join(", ")
    }

}

struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn new(count: u8) -> Deck {
        let mut cards = Vec::new();
        for _ in 0..count {    
            for &suit in &[Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
                for rank in 1..=13 {
                    cards.push(Card { rank, suit });
                }
            }
        }

        Deck { cards }
    }

    fn shuffle(&mut self) {
        let mut rng = rand::rng();
        for i in (1..self.cards.len()).rev() {
            let j = rng.random_range(0..=i);
            self.cards.swap(i, j);
        }
    }

    fn deal_card(&mut self) -> Option<Card> {
        self.cards.pop()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum GameResult {
    PlayerWin,
    DealerWin,
    Push,
    PlayerBlackjack,
}

struct BlackjackApp {
    last_game_result: Option<GameResult>,
    games_played: u32,
    wins: u32,
    losses: u32,
    pushes: u32,
    deck: Deck,
    bankroll: f64,
    bet_amount: f64,
}

impl Default for BlackjackApp {
    fn default() -> Self {
        let mut new_deck = Deck::new(6);
        new_deck.shuffle();
        Self {
            last_game_result: None,            
            games_played: 0,
            wins: 0,
            losses: 0,
            pushes: 0,
            deck: new_deck,
            bankroll: 1000.0,
            bet_amount: 10.0,
        }
    }
}

impl BlackjackApp {
    fn play_game(&mut self) {
        // Placeholder cut off of 15 cards to reshuffle
        if self.deck.cards.len() < 15 {
            self.deck = Deck::new(6);
            self.deck.shuffle();
        }

        let mut player_hand = Hand::new();
        let mut dealer_hand = Hand::new();

        player_hand.add_card(self.deck.deal_card().unwrap());
        dealer_hand.add_card(self.deck.deal_card().unwrap());
        player_hand.add_card(self.deck.deal_card().unwrap());
        dealer_hand.add_card(self.deck.deal_card().unwrap());



        
        if player_hand.is_blackjack() && dealer_hand.is_blackjack() {
            self.last_game_result = Some(GameResult::Push);
            self.pushes += 1;
            self.games_played += 1;
            let log = format!("*** Game {} ***\nPlayer's hand: {} (Total: {})\nDealer's hand: {} (Total: {})\nBoth have Blackjack! Push!\n", 
                self.games_played, player_hand.display(), player_hand.total(), dealer_hand.display(), dealer_hand.total());
            self.append_log(&log);
            return;
        } else if dealer_hand.is_blackjack() {
            self.last_game_result = Some(GameResult::DealerWin);
            self.losses += 1;
            self.games_played += 1;
            let log = format!("*** Game {} ***\nPlayer's hand: {} (Total: {})\nDealer's hand: {} (Total: {})\nBlackjack! Dealer wins!\n", 
                self.games_played, player_hand.display(), player_hand.total(), dealer_hand.display(), dealer_hand.total());
            self.append_log(&log);
            self.pay_bet(&GameResult::DealerWin);
            return;
        } else if player_hand.is_blackjack() {
            self.last_game_result = Some(GameResult::PlayerBlackjack);
            self.wins += 1;
            self.games_played += 1;
            let log = format!("*** Game {} ***\nPlayer's hand: {} (Total: {})\nDealer shows: {}\nBlackjack! Player wins!\n", 
                self.games_played, player_hand.display(), player_hand.total(), dealer_hand.cards[0].name());
            self.append_log(&log);
            self.pay_bet(&GameResult::PlayerBlackjack);
            return;
        }

        let mut log = String::new();
        log.push_str(&format!("*** Game {} ***\n", self.games_played + 1));
        log.push_str(&format!("Player's hand: {} (Total: {})\n", player_hand.display(), player_hand.total()));
        log.push_str(&format!("Dealer shows: {}\n", dealer_hand.cards[0].name()));

        while player_hand.total() < 17 {
            player_hand.add_card(self.deck.deal_card().unwrap());
            log.push_str(&format!("Player hits: {} (Total: {})\n", player_hand.cards.last().unwrap().name(), player_hand.total()));
            if player_hand.is_busted() {
                log.push_str("Player busts!\n");
                self.last_game_result = Some(GameResult::DealerWin);
                self.losses += 1;
                self.games_played += 1;
                self.append_log(&log);
                self.pay_bet(&GameResult::DealerWin);
                return;
            }                       
        }
        log.push_str("Player stands.\n");
        while dealer_hand.total() < 17 {
            dealer_hand.add_card(self.deck.deal_card().unwrap());
            log.push_str(&format!("Dealer hits: {} (Total: {})\n", dealer_hand.cards.last().unwrap().name(), dealer_hand.total()));
            if dealer_hand.is_busted() {
                log.push_str("Dealer busts!\n");
                self.last_game_result = Some(GameResult::PlayerWin);
                self.wins += 1;
                self.games_played += 1;
                self.append_log(&log);
                self.pay_bet(&GameResult::PlayerWin);
                return;
            }
        }
        log.push_str("Dealer stands.\n");
        log.push_str(&format!("Dealer's hand: {} (Total: {})\n", dealer_hand.display(), dealer_hand.total()));
        if player_hand.total() > dealer_hand.total() {
            log.push_str("Player wins!\n");
            self.last_game_result = Some(GameResult::PlayerWin);
            self.wins += 1;
        } else if player_hand.total() < dealer_hand.total() {
            log.push_str("Dealer wins!\n");
            self.last_game_result = Some(GameResult::DealerWin);
            self.losses += 1;
        } else {
            log.push_str("Push!\n");
            self.last_game_result = Some(GameResult::Push);
            self.pushes += 1;
        }
        self.games_played += 1;        
        self.append_log(&log);
        self.pay_bet(&self.last_game_result.clone().unwrap());
    }

    fn append_log(&self, log: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("blackjack_log.txt")
            .unwrap();
        writeln!(file, "{}", log).unwrap();
    }

    fn pay_bet(&mut self, result: &GameResult) {
        match result {
            GameResult::PlayerWin => self.bankroll += self.bet_amount,
            GameResult::DealerWin => self.bankroll -= self.bet_amount,
            GameResult::Push => {},
            GameResult::PlayerBlackjack => self.bankroll += self.bet_amount * 1.5,
        }
    }    
}

impl eframe::App for BlackjackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let can_play = self.bankroll >= self.bet_amount;
            ui.heading("Blackjack Simulator");
            if ui.add_enabled(can_play, egui::Button::new("Play Game")).clicked() {
                self.play_game();
            }
            if ui.add_enabled(can_play, egui::Button::new("Play 1000 Games")).clicked() {
                for _ in 0..1000 {
                    if self.bankroll < self.bet_amount {
                    ui.label("Insufficient bankroll to continue playing.");
                    return;
                }
                    self.play_game();
                }
            }
            if ui.button("Reset Bankroll").clicked() {
                self.bankroll = 1000.0;
                self.games_played = 0;
                self.wins = 0;
                self.losses = 0;
                self.pushes = 0;
                self.last_game_result = None;
            }
            if let Some(result) = &self.last_game_result {
                let result_str = match result {
                    GameResult::PlayerWin => "Player Wins!",
                    GameResult::DealerWin => "Dealer Wins!",
                    GameResult::Push => "Push!",
                    GameResult::PlayerBlackjack => "Player Wins with Blackjack!",
                };
                ui.label(format!("Last Game Result: {}", result_str));
            } else {
                ui.label("No games played yet.");
            }
            ui.separator();
            ui.label(format!("Bankroll: ${:.2}", self.bankroll));
            ui.label(format!("Games Played: {}", self.games_played));
            ui.label(format!("Wins: {}", self.wins));
            ui.label(format!("Losses: {}", self.losses));
            ui.label(format!("Pushes: {}", self.pushes));
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Blackjack Simulator",
        options,
        Box::new(|_cc| Ok(Box::<BlackjackApp>::default())),
    )
}