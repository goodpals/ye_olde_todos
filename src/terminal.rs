use terminal_size::{Height, Width, terminal_size};

const FALLBACK_WIDTH: usize = 200;

// Falls back to 200 if the terminal width is not available, e.g. in piped output.
pub fn get_terminal_width() -> usize {
    get_terminal_width_internal(terminal_size())
}

fn get_terminal_width_internal(size: Option<(Width, Height)>) -> usize {
    size.map(|(w, _)| w.0 as usize).unwrap_or(FALLBACK_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_terminal_width_with_value() {
        let mock_size = Some((Width(120), Height(40)));
        assert_eq!(get_terminal_width_internal(mock_size), 120);
    }
    #[test]
    fn test_get_terminal_width_fallback() {
        assert_eq!(get_terminal_width_internal(None), FALLBACK_WIDTH);
    }
}
