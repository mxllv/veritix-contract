pub fn require_positive_amount(amount: i128) {
    assert!(amount > 0, "Amount must be strictly positive");
}
