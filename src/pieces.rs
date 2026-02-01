// Traits and business logic for chess pieces.
// Piece structs wrap proto messages and implement the Piece trait.

use crate::rchess::v1::{self as proto};
use std::fmt;

/// Represents a square on the chessboard using file (column) and rank (row).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub file: u8, // 0..=7 (a..=h)
    pub rank: u8, // 0..=7 (1..=8)
}

impl Square {
    /// Create a new square from file and rank (0-indexed).
    pub fn new(file: u8, rank: u8) -> Option<Self> {
        if file <= 7 && rank <= 7 {
            Some(Square { file, rank })
        } else {
            None
        }
    }

    /// Create from a proto Position.
    pub fn from_proto(pos: &proto::Position) -> Option<Self> {
        let file = (pos.file as u8).saturating_sub(1); // proto file is 1-indexed
        let rank = (pos.rank as u8).saturating_sub(1); // proto rank is 1-indexed
        Square::new(file, rank)
    }

    /// Convert to proto Position.
    pub fn to_proto(&self) -> proto::Position {
        proto::Position {
            file: (self.file + 1) as i32, // convert to 1-indexed
            rank: (self.rank + 1) as i32,
            index: (self.rank * 8 + self.file) as i32,
            algebraic: self.to_algebraic(),
        }
    }

    /// Convert to algebraic notation (e.g., "e4").
    pub fn to_algebraic(&self) -> String {
        format!(
            "{}{}",
            (b'a' + self.file) as char,
            self.rank + 1
        )
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_algebraic())
    }
}

/// Color of a piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Return the opposite color.
    pub fn opposite(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Convert from proto Color.
    pub fn from_proto(proto_color: i32) -> Self {
        match proto_color {
            1 => Color::White,
            2 => Color::Black,
            _ => Color::White, // default
        }
    }

    /// Convert to proto Color.
    pub fn to_proto(&self) -> i32 {
        match self {
            Color::White => 1,
            Color::Black => 2,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

/// Piece type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PieceType::King => write!(f, "King"),
            PieceType::Queen => write!(f, "Queen"),
            PieceType::Rook => write!(f, "Rook"),
            PieceType::Bishop => write!(f, "Bishop"),
            PieceType::Knight => write!(f, "Knight"),
            PieceType::Pawn => write!(f, "Pawn"),
        }
    }
}

/// Core trait for all chess pieces.
pub trait Piece: fmt::Debug + Send + Sync {
    /// Return the color of the piece.
    fn color(&self) -> Color;

    /// Return the current square the piece is on.
    fn position(&self) -> Square;

    /// Return a type identifier for the piece.
    fn piece_type(&self) -> PieceType;

    /// Check if this piece can move to the given square (ignoring other pieces on the board).
    fn can_move_to(&self, target: Square) -> bool;

    /// Get all valid moves for this piece given the current board state.
    /// Considers piece blocking, pinning, check, castling legality, etc.
    fn valid_moves(&self, board: &crate::board::Board) -> Vec<Square>;

    /// Check if a specific move to target is valid given the current board state.
    fn is_valid_move(&self, target: Square, board: &crate::board::Board) -> bool {
        self.valid_moves(board).contains(&target)
    }

    /// Return a human-readable name (e.g., "White King").
    fn display_name(&self) -> String {
        format!("{} {}", self.color(), self.piece_type())
    }
}

/// King piece wrapping proto::King.
#[derive(Debug, Clone)]
pub struct King {
    inner: proto::King,
}

impl King {
    pub fn new(color: Color, position: Square) -> Self {
        King {
            inner: proto::King {
                color: color.to_proto(),
                position: Some(position.to_proto()),
                has_moved: false,
            },
        }
    }

    pub fn from_proto(proto: proto::King) -> Self {
        King { inner: proto }
    }

    pub fn to_proto(&self) -> proto::King {
        self.inner.clone()
    }

    pub fn has_moved(&self) -> bool {
        self.inner.has_moved
    }

    pub fn mark_moved(&mut self) {
        self.inner.has_moved = true;
    }
}

impl Piece for King {
    fn color(&self) -> Color {
        Color::from_proto(self.inner.color)
    }

    fn position(&self) -> Square {
        self.inner
            .position
            .as_ref()
            .and_then(Square::from_proto)
            .unwrap_or_else(|| Square::new(0, 0).unwrap())
    }

    fn piece_type(&self) -> PieceType {
        PieceType::King
    }

    fn can_move_to(&self, target: Square) -> bool {
        let pos = self.position();
        let file_diff = (pos.file as i32 - target.file as i32).abs();
        let rank_diff = (pos.rank as i32 - target.rank as i32).abs();
        (file_diff <= 1 && rank_diff <= 1) && !(file_diff == 0 && rank_diff == 0)
    }

    fn valid_moves(&self, board: &crate::board::Board) -> Vec<Square> {
        let mut moves = Vec::new();
        let pos = self.position();

        for file in 0..=7 {
            for rank in 0..=7 {
                if let Some(target) = Square::new(file, rank) {
                    if self.can_move_to(target) && board.is_empty_or_capturable(target, self.color()) {
                        moves.push(target);
                    }
                }
            }
        }
        moves
    }
}

/// Queen piece wrapping proto::Queen.
#[derive(Debug, Clone)]
pub struct Queen {
    inner: proto::Queen,
}

impl Queen {
    pub fn new(color: Color, position: Square) -> Self {
        Queen {
            inner: proto::Queen {
                color: color.to_proto(),
                position: Some(position.to_proto()),
            },
        }
    }

    pub fn from_proto(proto: proto::Queen) -> Self {
        Queen { inner: proto }
    }

    pub fn to_proto(&self) -> proto::Queen {
        self.inner.clone()
    }
}

impl Piece for Queen {
    fn color(&self) -> Color {
        Color::from_proto(self.inner.color)
    }

    fn position(&self) -> Square {
        self.inner
            .position
            .as_ref()
            .and_then(Square::from_proto)
            .unwrap_or_else(|| Square::new(0, 0).unwrap())
    }

    fn piece_type(&self) -> PieceType {
        PieceType::Queen
    }

    fn can_move_to(&self, target: Square) -> bool {
        let pos = self.position();
        let file_diff = (pos.file as i32 - target.file as i32).abs();
        let rank_diff = (pos.rank as i32 - target.rank as i32).abs();
        (file_diff == 0 || rank_diff == 0 || file_diff == rank_diff)
            && !(file_diff == 0 && rank_diff == 0)
    }

    fn valid_moves(&self, board: &crate::board::Board) -> Vec<Square> {
        board.sliding_piece_moves(self.position(), self.color(), &[
            (0, 1), (0, -1), (1, 0), (-1, 0), // orthogonal
            (1, 1), (1, -1), (-1, 1), (-1, -1), // diagonal
        ])
    }
}

/// Rook piece wrapping proto state.
#[derive(Debug, Clone)]
pub struct Rook {
    color: Color,
    position: Square,
    has_moved: bool,
}

impl Rook {
    pub fn new(color: Color, position: Square) -> Self {
        Rook {
            color,
            position,
            has_moved: false,
        }
    }

    pub fn has_moved(&self) -> bool {
        self.has_moved
    }

    pub fn mark_moved(&mut self) {
        self.has_moved = true;
    }
}

impl Piece for Rook {
    fn color(&self) -> Color {
        self.color
    }

    fn position(&self) -> Square {
        self.position
    }

    fn piece_type(&self) -> PieceType {
        PieceType::Rook
    }

    fn can_move_to(&self, target: Square) -> bool {
        let file_diff = (self.position.file as i32 - target.file as i32).abs();
        let rank_diff = (self.position.rank as i32 - target.rank as i32).abs();
        (file_diff == 0 || rank_diff == 0) && !(file_diff == 0 && rank_diff == 0)
    }

    fn valid_moves(&self, board: &crate::board::Board) -> Vec<Square> {
        board.sliding_piece_moves(self.position, self.color, &[
            (0, 1), (0, -1), (1, 0), (-1, 0),
        ])
    }
}

/// Bishop square color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BishopSquareColor {
    Light,
    Dark,
}

impl BishopSquareColor {
    fn to_proto(&self) -> i32 {
        match self {
            BishopSquareColor::Light => 1,
            BishopSquareColor::Dark => 2,
        }
    }

    fn from_proto(val: i32) -> Self {
        match val {
            1 => BishopSquareColor::Light,
            2 => BishopSquareColor::Dark,
            _ => BishopSquareColor::Light,
        }
    }
}

/// Bishop piece wrapping proto::Bishop.
#[derive(Debug, Clone)]
pub struct Bishop {
    inner: proto::Bishop,
}

impl Bishop {
    pub fn new(color: Color, position: Square, square_color: BishopSquareColor) -> Self {
        Bishop {
            inner: proto::Bishop {
                color: color.to_proto(),
                position: Some(position.to_proto()),
                square_color: square_color.to_proto(),
            },
        }
    }

    pub fn from_proto(proto: proto::Bishop) -> Self {
        Bishop { inner: proto }
    }

    pub fn to_proto(&self) -> proto::Bishop {
        self.inner.clone()
    }

    pub fn square_color(&self) -> BishopSquareColor {
        BishopSquareColor::from_proto(self.inner.square_color)
    }
}

impl Piece for Bishop {
    fn color(&self) -> Color {
        Color::from_proto(self.inner.color)
    }

    fn position(&self) -> Square {
        self.inner
            .position
            .as_ref()
            .and_then(Square::from_proto)
            .unwrap_or_else(|| Square::new(0, 0).unwrap())
    }

    fn piece_type(&self) -> PieceType {
        PieceType::Bishop
    }

    fn can_move_to(&self, target: Square) -> bool {
        let pos = self.position();
        let file_diff = (pos.file as i32 - target.file as i32).abs();
        let rank_diff = (pos.rank as i32 - target.rank as i32).abs();
        file_diff == rank_diff && file_diff != 0
    }

    fn valid_moves(&self, board: &crate::board::Board) -> Vec<Square> {
        board.sliding_piece_moves(self.position(), self.color(), &[
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ])
    }
}

/// Knight piece wrapping proto::Knight.
#[derive(Debug, Clone)]
pub struct Knight {
    inner: proto::Knight,
}

impl Knight {
    pub fn new(color: Color, position: Square) -> Self {
        Knight {
            inner: proto::Knight {
                color: color.to_proto(),
                position: Some(position.to_proto()),
            },
        }
    }

    pub fn from_proto(proto: proto::Knight) -> Self {
        Knight { inner: proto }
    }

    pub fn to_proto(&self) -> proto::Knight {
        self.inner.clone()
    }
}

impl Piece for Knight {
    fn color(&self) -> Color {
        Color::from_proto(self.inner.color)
    }

    fn position(&self) -> Square {
        self.inner
            .position
            .as_ref()
            .and_then(Square::from_proto)
            .unwrap_or_else(|| Square::new(0, 0).unwrap())
    }

    fn piece_type(&self) -> PieceType {
        PieceType::Knight
    }

    fn can_move_to(&self, target: Square) -> bool {
        let pos = self.position();
        let file_diff = (pos.file as i32 - target.file as i32).abs();
        let rank_diff = (pos.rank as i32 - target.rank as i32).abs();
        (file_diff == 2 && rank_diff == 1) || (file_diff == 1 && rank_diff == 2)
    }

    fn valid_moves(&self, board: &crate::board::Board) -> Vec<Square> {
        let mut moves = Vec::new();
        let pos = self.position();

        let offsets = [
            (2, 1), (2, -1), (-2, 1), (-2, -1),
            (1, 2), (1, -2), (-1, 2), (-1, -2),
        ];

        for (df, dr) in offsets {
            if let Some(target) = Square::new(
                (pos.file as i32 + df) as u8,
                (pos.rank as i32 + dr) as u8,
            ) {
                if board.is_empty_or_capturable(target, self.color()) {
                    moves.push(target);
                }
            }
        }
        moves
    }
}

/// Pawn piece wrapping proto::Pawn.
#[derive(Debug, Clone)]
pub struct Pawn {
    inner: proto::Pawn,
}

impl Pawn {
    pub fn new(color: Color, position: Square) -> Self {
        Pawn {
            inner: proto::Pawn {
                color: color.to_proto(),
                position: Some(position.to_proto()),
                has_moved: false,
                promoted_to: 0,
                en_passant_vulnerable: false,
            },
        }
    }

    pub fn from_proto(proto: proto::Pawn) -> Self {
        Pawn { inner: proto }
    }

    pub fn to_proto(&self) -> proto::Pawn {
        self.inner.clone()
    }

    pub fn has_moved(&self) -> bool {
        self.inner.has_moved
    }

    pub fn mark_moved(&mut self) {
        self.inner.has_moved = true;
    }

    pub fn promoted_to(&self) -> Option<PieceType> {
        match self.inner.promoted_to {
            1 => Some(PieceType::King),
            2 => Some(PieceType::Queen),
            3 => Some(PieceType::Rook),
            4 => Some(PieceType::Bishop),
            5 => Some(PieceType::Knight),
            6 => Some(PieceType::Pawn),
            _ => None,
        }
    }

    pub fn set_promoted_to(&mut self, piece_type: PieceType) {
        self.inner.promoted_to = match piece_type {
            PieceType::King => 1,
            PieceType::Queen => 2,
            PieceType::Rook => 3,
            PieceType::Bishop => 4,
            PieceType::Knight => 5,
            PieceType::Pawn => 6,
        };
    }

    pub fn en_passant_vulnerable(&self) -> bool {
        self.inner.en_passant_vulnerable
    }

    pub fn set_en_passant_vulnerable(&mut self, vulnerable: bool) {
        self.inner.en_passant_vulnerable = vulnerable;
    }
}

impl Piece for Pawn {
    fn color(&self) -> Color {
        Color::from_proto(self.inner.color)
    }

    fn position(&self) -> Square {
        self.inner
            .position
            .as_ref()
            .and_then(Square::from_proto)
            .unwrap_or_else(|| Square::new(0, 0).unwrap())
    }

    fn piece_type(&self) -> PieceType {
        PieceType::Pawn
    }

    fn can_move_to(&self, target: Square) -> bool {
        let pos = self.position();
        let direction = match self.color() {
            Color::White => 1i32,
            Color::Black => -1i32,
        };

        let rank_diff = target.rank as i32 - pos.rank as i32;
        let file_diff = (target.file as i32 - pos.file as i32).abs();

        if file_diff == 0 {
            if rank_diff == direction {
                true
            } else if rank_diff == direction * 2 && !self.has_moved() {
                true
            } else {
                false
            }
        } else if file_diff == 1 && rank_diff == direction {
            true
        } else {
            false
        }
    }

    fn valid_moves(&self, board: &crate::board::Board) -> Vec<Square> {
        board.pawn_moves(self.position(), self.color(), self.has_moved())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square_creation() {
        let sq = Square::new(4, 3).unwrap();
        assert_eq!(sq.file, 4);
        assert_eq!(sq.rank, 3);
        assert_eq!(sq.to_algebraic(), "e4");
    }

    #[test]
    fn test_color_opposite() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn test_king_movement() {
        let king = King::new(Color::White, Square::new(4, 4).unwrap());
        assert!(king.can_move_to(Square::new(4, 5).unwrap()));
        assert!(king.can_move_to(Square::new(5, 5).unwrap()));
        assert!(!king.can_move_to(Square::new(4, 6).unwrap()));
    }

    #[test]
    fn test_knight_movement() {
        let knight = Knight::new(Color::White, Square::new(4, 4).unwrap());
        assert!(knight.can_move_to(Square::new(5, 6).unwrap()));
        assert!(knight.can_move_to(Square::new(6, 5).unwrap()));
        assert!(!knight.can_move_to(Square::new(5, 5).unwrap()));
    }

    #[test]
    fn test_pawn_initial_move() {
        let pawn = Pawn::new(Color::White, Square::new(4, 1).unwrap());
        assert!(pawn.can_move_to(Square::new(4, 2).unwrap()));
        assert!(pawn.can_move_to(Square::new(4, 3).unwrap()));
    }

    #[test]
    fn test_pawn_after_first_move() {
        let mut pawn = Pawn::new(Color::White, Square::new(4, 1).unwrap());
        pawn.mark_moved();
        assert!(pawn.can_move_to(Square::new(4, 2).unwrap()));
        assert!(!pawn.can_move_to(Square::new(4, 3).unwrap()));
    }

    #[test]
    fn test_bishop_square_color() {
        let bishop = Bishop::new(Color::White, Square::new(2, 0).unwrap(), BishopSquareColor::Light);
        assert_eq!(bishop.square_color(), BishopSquareColor::Light);
    }
}
