#  cw-payment-splitter

This [CosmWasm](https://www.cosmwasm.com/) smart contract allows splitting payments across multiple addresses based on the number of shares associated with each address. It takes a list of addresses and a list of shares upon initialization. The funds that the contract instance receives can then be released to an address based on the percentage of shares associated with the address requested.

### Instantiate

Create new instance of `cw-payment-splitter` contract.

```
wasmd tx wasm instantiate <code_id> '{"payees":["<address>", ["<address>"], "shares":"[1, 2]"}' --from <creator_address> --label="<label>" --gas="auto" --chain-id="<chain_id>"
```


### Execute Release

Release funds for payee `address` of `cw-payment-splitter` contract instance.

```
wasmd tx wasm execute <payment_splitter_contract_address> '{"release":{"address":"<address>"}}' --from <address> --chain-id="<chain_id>"
```

### Query Payees

Get array of payee addresses for `cw-payment-splitter` contract instance.

```
wasmd query wasm contract-state smart <payment_splitter_contract_address> '{"get_payees":{}}' --chain-id="<chain_id>"
```

### Query Released

Get amount released to payee `address` for `cw-payment-splitter` contract instance.

```
wasmd query wasm contract-state smart <payment_splitter_contract_address> '{"get_released":{"address": "<address>"}}' --chain-id="<chain_id>"
```

### Query Shares

Get number of shares associated with payee `address` for `cw-payment-splitter` contract instance.

```
wasmd query wasm contract-state smart <payment_splitter_contract_address> '{"get_shares":{"address": "<address>"}}' --chain-id="<chain_id>"
```

### Query Total Released

Get total amount released to all payee addresses for `cw-payment-splitter` contract instance.

```
wasmd query wasm contract-state smart <payment_splitter_contract_address> '{"get_total_released":{}}' --chain-id="<chain_id>"
```

### Query Total Shares

Get total number of shares for all payee addresses for `cw-payment-splitter` contract instance.

```
wasmd query wasm contract-state smart <payment_splitter_contract_address> '{"get_total_shares":{}}' --chain-id="<chain_id>"
```
