//! Kinesin's owned execution boundaries.

pub fn product_name() -> &'static str {
    "Kinesin"
}

#[cfg(test)]
mod tests {
    #[test]
    fn binary_entry_uses_the_library() {
        assert_eq!(super::product_name(), "Kinesin");
    }
}
