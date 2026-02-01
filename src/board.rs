// Board state and move validation logic.
// Board struct wraps proto GameState and provides efficient indices for piece lookups.

use crate::pieces::{Color, Square};
use crate::rchess::v1::{self as proto};
use std::collections::HashMap;

/// Board wraps proto GameState and provides efficient piece lookup and move validation.
#[derive(Debug, Clone)]
pub struct Board {
    inner: proto::GameState,
    // Efficient index: Square → Piece (cached from inner.board.pieces)
    square_to_piece: HashMap<Square, proto::Piece>,
    // Cached lists of pieces by color for quick filtering
    white_pieces: Vec<proto::Piece>,
    black_pieces: Vec<proto::Piece>,
}

impl Board {
    /// Create a new board from a GameState proto, building indices.
    pub fn from_proto(proto: proto::GameState) -> Self {
        let mut board = Board {
            inner: proto,
            square_to_piece: HashMap::new(),
            white_pieces: Vec::new(),
            black_pieces: Vec::new(),
        };
        board.rebuild_indices();
        board
    }

    /// Convert back to proto GameState.
    pub fn to_proto(&self) -> proto::GameState {
        self.inner.clone()
    }

    /// Rebuild internal indices from the proto pieces list.
    /// Call this after modifying the pieces.
    fn rebuild_indices(&mut self) {
        self.square_to_piece.clear();
        self.white_pieces.clear();
        self.black_pieces.clear();

        if let Some(board) = &self.inner.board {
            for piece in &board.pieces {
                if piece.captured {
                    continue;
                }

                // Add to square-to-piece map
                if let Some(square) = self.piece_square(piece) {
                    self.square_to_piece.insert(square, piece.clone());
                }

                // Add to color-filtered lists
                if let Some(color) = self.piece_color(piece) {
                    match color {
                        Color::White => self.white_pieces.push(piece.clone()),
                        Color::Black => self.black_pieces.push(piece.clone()),
                    }
                }
            }
        }
    }

    /// Get the piece at a given square, if any.
    pub fn piece_at(&self, square: Square) -> Option<&proto::Piece> {
        self.square_to_piece.get(&square)
    }

    /// Check if a square is empty or contains an opponent's piece.
    pub fn is_empty_or_capturable(&self, square: Square, color: Color) -> bool {
        if let Some(piece) = self.piece_at(square) {
            // Square has a piece; check if it's an opponent
            let piece_color = self.piece_color(piece);
            piece_color != Some(color)
        } else {
            // Square is empty
            true
        }
    }

    /// Get all pieces of a given color.
    pub fn pieces_of_color(&self, color: Color) -> &[proto::Piece] {
        match color {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces,
        }
    }

    /// Get all non-captured pieces.
    pub fn all_pieces(&self) -> impl Iterator<Item = &proto::Piece> {
        self.square_to_piece.values()
    }

    /// Get the color of a piece from its proto representation.
    fn piece_color(&self, piece: &proto::Piece) -> Option<Color> {
        if let Some(kind) = &piece.kind {
            match kind {
                proto::piece::Kind::King(k) => Some(Color::from_proto(k.color)),
                proto::piece::Kind::Queen(q) => Some(Color::from_proto(q.color)),
                proto::piece::Kind::Knight(n) => Some(Color::from_proto(n.color)),
                proto::piece::Kind::Bishop(b) => Some(Color::from_proto(b.color)),
                proto::piece::Kind::Pawn(p) => Some(Color::from_proto(p.color)),
            }
        } else {
            None
        }
    }

    /// Get the square of a piece from its proto representation.
    fn piece_square(&self, piece: &proto::Piece) -> Option<Square> {
        if let Some(kind) = &piece.kind {
            match kind {
                proto::piece::Kind::King(k) => k.position.as_ref().and_then(Square::from_proto),
                proto::piece::Kind::Queen(q) => q.position.as_ref().and_then(Square::from_proto),
                proto::piece::Kind::Knight(n) => n.position.as_ref().and_then(Square::from_proto),
                proto::piece::Kind::Bishop(b) => b.position.as_ref().and_then(Square::from_proto),
                proto::piece::Kind::Pawn(p) => p.position.as_ref().and_then(Square::from_proto),
            }
        } else {
            None
        }
    }

    /// Get all valid moves for a sliding piece (queen, rook, bishop) in given directions.
    pub fn sliding_piece_moves(
        &self,
        from: Square,
        color: Color,
        directions: &[(i32, i32)],
    ) -> Vec<Square> {
        let mut moves = Vec::new();

        for &(df, dr) in directions {
            let mut file = from.file as i32;
            let mut rank = from.rank as i32;

            loop {
                file += df;
                rank += dr;

                if file < 0 || file > 7 || rank < 0 || rank > 7 {
                    break;
                }

                if let Some(target) = Square::new(file as u8, rank as u8) {
                    if self.is_empty_or_capturable(target, color) {
                        moves.push(target);
                        // If there's an opponent piece, stop sliding in this direction
                        if let Some(piece) = self.piece_at(target) {
                            if self.piece_color(piece) != Some(color) {
                                break;
                            }
                        }
                    } else {
                        // Square occupied by own piece, stop sliding
                        break;
                    }
                }
            }
        }

        moves
    }

    /// Get all valid pawn moves from a given square.
    pub fn pawn_moves(&self, from: Square, color: Color, has_moved: bool) -> Vec<Square> {
        let mut moves = Vec::new();
        let direction = match color {
            Color::White => 1i32,
            Color::Black => -1i32,
        };

        // Forward moves
        if let Some(target) = Square::new(
            from.file,
            (from.rank as i32 + direction) as u8,
        ) {
            if self.piece_at(target).is_none() {
                moves.push(target);

                // Two-square move from starting position
                if !has_moved {
                    if let Some(two_sq) = Square::new(
                        from.file,
                        (from.rank as i32 + 2 * direction) as u8,
                    ) {
                        if self.piece_at(two_sq).is_none() {
                            moves.push(two_sq);
                        }
                    }
                }
            }
        }

        // Capture moves
        for &df in &[-1i32, 1i32] {
            if let Some(target) = Square::new(
                (from.file as i32 + df) as u8,
                (from.rank as i32 + direction) as u8,
            ) {
                if let Some(piece) = self.piece_at(target) {
                    if self.piece_color(piece) == Some(color.opposite()) {
                        moves.push(target);
                    }
                }
                // TODO: En-passant capture
            }
        }

        moves
    }

    /// Get current player color.
    pub fn current_player(&self) -> Color {
        Color::from_proto(self.inner.current_player)
    }

    /// Get castling rights.
    pub fn white_kingside_castling(&self) -> bool {
        self.inner.white_kingside_castling
    }

    pub fn white_queenside_castling(&self) -> bool {
        self.inner.white_queenside_castling
    }

    pub fn black_kingside_castling(&self) -> bool {
        self.inner.black_kingside_castling
    }

    pub fn black_queenside_castling(&self) -> bool {
        self.inner.black_queenside_castling
    }

    /// Get en-passant target square, if any.
    pub fn en_passant_target(&self) -> Option<Square> {
        self.inner
            .en_passant_target
            .as_ref()
            .and_then(Square::from_proto)
    }

    /// Get halfmove clock (for fifty-move rule).
    pub fn halfmove_clock(&self) -> i32 {
        self.inner.halfmove_clock
    }

    /// Get fullmove number.
    pub fn fullmove_number(&self) -> i32 {
        self.inner.fullmove_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_creation_empty() {
        let game_state = proto::GameState {
            board: Some(proto::Board::default()),
            current_player: 1, // White
            ..Default::default()
        };
        let board = Board::from_proto(game_state);
        assert_eq!(board.all_pieces().count(), 0);
        assert_eq!(board.pieces_of_color(Color::White).len(), 0);
        assert_eq!(board.pieces_of_color(Color::Black).len(), 0);
    }

    #[test]
    fn test_piece_at_empty_square() {
        let game_state = proto::GameState {
            board: Some(proto::Board::default()),
            ..Default::default()
        };
        let board = Board::from_proto(game_state);
        let sq = Square::new(4, 4).unwrap();
        assert!(board.piece_at(sq).is_none());
    }

    #[test]
    fn test_empty_or_capturable() {
        let game_state = proto::GameState {
            board: Some(proto::Board::default()),
            ..Default::default()
        };
        let board = Board::from_proto(game_state);
        let sq = Square::new(4, 4).unwrap();
        assert!(board.is_empty_or_capturable(sq, Color::White));
        assert!(board.is_empty_or_capturable(sq, Color::Black));
    }

    #[test]
    fn test_current_player() {
        let game_state = proto::GameState {
            board: Some(proto::Board::default()),
            current_player: 1, // White
            ..Default::default()
        };
        let board = Board::from_proto(game_state);
        assert_eq!(board.current_player(), Color::White);
    }
}
