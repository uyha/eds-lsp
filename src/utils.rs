use tower_lsp::lsp_types::Position;
use tree_sitter::Point;

use tower_lsp::lsp_types;
use tree_sitter;

pub fn point_to_position(point: &Point) -> Position {
    Position {
        line: point.row as u32,
        character: point.column as u32,
    }
}

pub fn position_to_point(position: &Position) -> Point {
    Point {
        row: position.line as usize,
        column: position.character as usize,
    }
}

pub fn ts_to_lsp_range(range: tree_sitter::Range) -> lsp_types::Range {
    lsp_types::Range {
        start: point_to_position(&range.start_point),
        end: point_to_position(&range.end_point),
    }
}
