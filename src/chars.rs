/// characters from the box drawing character set.
///
/// see <https://www.unicode.org/charts/nameslist/n_2500.html>.
#[allow(dead_code, reason = "constants")]
pub mod box_drawing {
    pub mod light {
        pub const HORIZONTAL: char = '\u{2500}'; //	 ─ 	Box Drawings Light Horizontal
        pub const VERTICAL: char = '\u{2502}'; //	 │ 	Box Drawings Light Vertical
        pub const TRIPLE_DASH_HORIZONTAL: char = '\u{2504}'; //	 ┄ 	Box Drawings Light Triple Dash Horizontal
        pub const TRIPLE_DASH_VERTICAL: char = '\u{2506}'; //	 ┆ 	Box Drawings Light Triple Dash Vertical
        pub const QUADRUPLE_DASH_HORIZONTAL: char = '\u{2508}'; //	 ┈ 	Box Drawings Light Quadruple Dash Horizontal
        pub const QUADRUPLE_DASH_VERTICAL: char = '\u{250A}'; //	 ┊ 	Box Drawings Light Quadruple Dash Vertical
        pub const DOWN_AND_RIGHT: char = '\u{250C}'; //	 ┌ 	Box Drawings Light Down And Right
        pub const DOWN_AND_LEFT: char = '\u{2510}'; //	 ┐ 	Box Drawings Light Down And Left
        pub const UP_AND_RIGHT: char = '\u{2514}'; //	 └ 	Box Drawings Light Up And Right
        pub const UP_AND_LEFT: char = '\u{2518}'; //	 ┘ 	Box Drawings Light Up And Left
        pub const VERTICAL_AND_RIGHT: char = '\u{251C}'; //	 ├ 	Box Drawings Light Vertical And Right
        pub const VERTICAL_AND_LEFT: char = '\u{2524}'; //	 ┤ 	Box Drawings Light Vertical And Left
        pub const DOWN_AND_HORIZONTAL: char = '\u{252C}'; //	 ┬ 	Box Drawings Light Down And Horizontal
        pub const UP_AND_HORIZONTAL: char = '\u{2534}'; //	 ┴ 	Box Drawings Light Up And Horizontal
        pub const VERTICAL_AND_HORIZONTAL: char = '\u{253C}'; //	 ┼ 	Box Drawings Light Vertical And Horizontal
        pub const DOUBLE_DASH_HORIZONTAL: char = '\u{254C}'; //	 ╌ 	Box Drawings Light Double Dash Horizontal
        pub const DOUBLE_DASH_VERTICAL: char = '\u{254E}'; //	 ╎ 	Box Drawings Light Double Dash Vertical
        pub const ARC_DOWN_AND_RIGHT: char = '\u{256D}'; //	 ╭ 	Box Drawings Light Arc Down And Right
        pub const ARC_DOWN_AND_LEFT: char = '\u{256E}'; //	 ╮ 	Box Drawings Light Arc Down And Left
        pub const ARC_UP_AND_LEFT: char = '\u{256F}'; //	 ╯ 	Box Drawings Light Arc Up And Left
        pub const ARC_UP_AND_RIGHT: char = '\u{2570}'; //	 ╰ 	Box Drawings Light Arc Up And Right
        pub const DIAGONAL_UPPER_RIGHT_TO_LOWER_LEFT: char = '\u{2571}'; //	 ╱ 	Box Drawings Light Diagonal Upper Right To Lower Left
        pub const DIAGONAL_UPPER_LEFT_TO_LOWER_RIGHT: char = '\u{2572}'; //	 ╲ 	Box Drawings Light Diagonal Upper Left To Lower Right
        pub const DIAGONAL_CROSS: char = '\u{2573}'; //	 ╳ 	Box Drawings Light Diagonal Cross
        pub const LEFT: char = '\u{2574}'; //	 ╴ 	Box Drawings Light Left
        pub const UP: char = '\u{2575}'; //	 ╵ 	Box Drawings Light Up
        pub const RIGHT: char = '\u{2576}'; //	 ╶ 	Box Drawings Light Right
        pub const DOWN: char = '\u{2577}'; //	 ╷ 	Box Drawings Light Down
        pub const LEFT_AND_HEAVY_RIGHT: char = '\u{257C}'; //	 ╼ 	Box Drawings Light Left And Heavy Right
        pub const UP_AND_HEAVY_DOWN: char = '\u{257D}'; //	 ╽ 	Box Drawings Light Up And Heavy Down
    }
}
