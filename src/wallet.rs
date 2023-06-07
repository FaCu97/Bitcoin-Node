pub struct Wallet {
    account: Vec<User>,
    node: Node,
    // current_account : User,
}

impl Wallet {
    pub fn make_transaction(account: User, adress: &str, value: i64) -> Result<(), &'static str> {
        if !account.has_balance(value) {
            return Err(
                "El balance de la cuenta {} tiene menos de {} satoshis",
                account.address,
                value,
            );
        }
        self.node.make_transaction(account.get_adress())
    }
}
