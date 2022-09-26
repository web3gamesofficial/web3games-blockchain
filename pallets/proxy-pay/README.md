## rpc
* get_amount_in_price: 给定代币，输出得到的另一种代币金额
```rust
    #[method(name = "exchange_getAmountInPrice")]
      fn get_amount_in_price(
        &self,
        supply: Balance,
        path: Vec<u128>,
        at: Option<BlockHash>,
      ) -> RpcResult<Option<Vec<Balance>>>;
```
input:
```json
{
  "id":1, 
  "jsonrpc":"2.0", 
  "method":"exchange_getAmountInPrice",
  "params":[
    10000000000,[3,2,1]
  ]
}
```
output:
```json
{
    "jsonrpc": "2.0",
    "result": [
        3376136670,
        5028088514,
        10000000000
    ],
    "id": 1
}
```

* get_amount_out_price:给定输出代币金额，获取输入金额
```rust
	#[method(name = "exchange_getAmountOutPrice")]
	fn get_amount_out_price(
		&self,
		supply: Balance,
		path: Vec<u128>,
		at: Option<BlockHash>,
	) -> RpcResult<Option<Vec<Balance>>>;
```
input:
```json
{
  "id":1,
  "jsonrpc":"2.0",
  "method":"exchange_getAmountOutPrice",
  "params":[
    10000000000,[1,2,3]
  ]
}
```
output:
```json
{
  "jsonrpc": "2.0",
  "result": [
    10000000000,
    4997965235,
    3325756438
  ],
  "id": 1
}
```

* get_estimate_lp_token:给定两个代币，输出预估得到的流动性
```rust
    #[method(name = "exchange_EstimateLpToken")]
    fn get_estimate_lp_token(
            &self,
            token_0: u128,
            amount_0: Balance,
            token_1: u128,
            amount_1: Balance,
            at: Option<BlockHash>,
    ) -> RpcResult<Option<Balance>>;
```
input:
```json
{
  "id":1,
  "jsonrpc":"2.0",
  "method":"exchange_EstimateLpToken",
  "params":[
    1,12096699998,2,10000000000
  ]
}
```
output:
```json
{
  "jsonrpc": "2.0",
  "result": 10996999997,
  "id": 1
}
```

* get_estimate_out_token:添加流动性时，给定一种代币金额，预估出另外一种代币金额，token_0/token_0，参数可以交换着填
```rust
    #[method(name = "exchange_EstimateOutToken")]
    fn get_estimate_out_token(
          &self,
          sopply: Balance,
          token_0: u128,
          token_1: u128,
          at: Option<BlockHash>,
    ) -> RpcResult<Option<Balance>>;
```
input:
```json
{
  "id":1,
  "jsonrpc":"2.0",
  "method":"exchange_EstimateOutToken",
  "params":[
    10000000000,2,1
  ]
}
```
output:
```json
{
  "jsonrpc": "2.0",
  "result": 12096699998,
  "id": 1
}
```

## rpc types
```json
 rpc: {
            "exchange": {
                "getAmountOutPrice": {
                    "description": "get amount out price",
                    "params": [
                        {
                            "name": "supply",
                            "type": "u128"
                        },
                        {
                            "name": "path",
                            "type": "Vec<u128>"
                        },
                        {
                            "name": "at",
                            "type": "Hash",
                            "isOptional": true
                        }
                    ],
                    "type": "Vec<u128>",
                },
                "getAmountInPrice": {
                    "description": "get amount in price",
                    "params": [
                        {
                            "name": "supply",
                            "type": "u128"
                        },
                        {
                            "name": "path",
                            "type": "Vec<u128>"
                        },
                        {
                            "name": "at",
                            "type": "Hash",
                            "isOptional": true
                        }
                    ],
                    "type": "Vec<u128>",
                },
                "getEstimateLpToken": {
                    "description": "get estimate lp token",
                    "params": [
                        {
                            "name": "token_0",
                            "type": "u128"
                        },
                        {
                            "name": "amount_0",
                            "type": "u128"
                        },
                        {
                            "name": "token_1",
                            "type": "u128"
                        },
                        {
                            "name": "amount_1",
                            "type": "u128"
                        },
                        {
                            "name": "at",
                            "type": "Hash",
                            "isOptional": true
                        }
                    ],
                    "type": "u128",
                },
                "getEstimateOutToken": {
                    "description": "get estimate out token",
                    "params": [
                        {
                            "name": "supply",
                            "type": "u128"
                        },
                        {
                            "name": "token_0",
                            "type": "u128"
                        },
                        {
                            "name": "token_1",
                            "type": "u128"
                        },
                        {
                            "name": "at",
                            "type": "Hash",
                            "isOptional": true
                        }
                    ],
                    "type": "u128",
                },
                "getLiquidityToTokens": {
                    "description": "get liquidity to tokens",
                    "params": [
                        {
                            "name": "lp_token",
                            "type": "u128"
                        },
                        {
                            "name": "lp_balance",
                            "type": "u128"
                        },
                        {
                            "name": "at",
                            "type": "Hash",
                            "isOptional": true
                        }
                    ],
                    "type": "(u128,u128)",
                },
            },
        }
```
