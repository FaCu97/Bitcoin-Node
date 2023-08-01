type Address = String;
type WifPrivateKey = String;
type AccountIndex = usize;
type Amount = i64;
type Fee = i64;
type BlockHash = [u8; 32];
type BlockHashString = String;
type TransactionHash = String;

pub enum WalletEvent {
    Start,
    AddAccountRequest(WifPrivateKey, Address),
    MakeTransaction(Address, Amount, Fee),
    PoiOfTransactionRequest(BlockHashString, TransactionHash),
    Finish,
    ChangeAccount(AccountIndex),
    GetAccountRequest,
    SearchBlock(BlockHash),
    SearchHeader(BlockHash),
}
