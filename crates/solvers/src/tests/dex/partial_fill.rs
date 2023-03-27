use {
    crate::tests::{self, balancer, mock},
    serde_json::json,
};

/// Tests that dex solvers consecutively decrease the amounts they try to fill
/// partially fillable orders with across `/solve` requests to eventually find a
/// fillable amount that works.
#[tokio::test]
async fn tested_amounts_decrease() {
    // shared::tracing::initialize_reentrant("solvers=trace");
    let inner_request = |amount| {
        json!({
            "sellToken": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            "buyToken": "0xba100000625a3754423978a60c9317c58a424e3d",
            "orderKind": "sell",
            "amount": amount,
            "gasPrice": "15000000000",
        })
    };

    let no_swap_found_response = json!({
        "tokenAddresses": [],
        "swaps": [],
        "swapAmount": "0",
        "swapAmountForSwaps": "0",
        "returnAmount": "0",
        "returnAmountFromSwaps": "0",
        "returnAmountConsideringFees": "0",
        "tokenIn": "0x0000000000000000000000000000000000000000",
        "tokenOut": "0x0000000000000000000000000000000000000000",
        "marketSp": "0",
    });

    let api = mock::http::setup(vec![
        mock::http::Expectation::Post {
            path: mock::http::Path::Any,
            req: inner_request("16000000000000000000"),
            res: no_swap_found_response.clone(),
        },
        mock::http::Expectation::Post {
            path: mock::http::Path::Any,
            req: inner_request("8000000000000000000"),
            res: no_swap_found_response.clone(),
        },
        mock::http::Expectation::Post {
            path: mock::http::Path::Any,
            req: inner_request("4000000000000000000"),
            res: no_swap_found_response.clone(),
        },
        mock::http::Expectation::Post {
            path: mock::http::Path::Any,
            req: inner_request("2000000000000000000"),
            res: no_swap_found_response.clone(),
        },
        mock::http::Expectation::Post {
            path: mock::http::Path::Any,
            req: inner_request("1000000000000000000"),
            res: json!({
                "tokenAddresses": [
                    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                    "0xba100000625a3754423978a60c9317c58a424e3d"
                ],
                "swaps": [
                    {
                        "poolId": "0x5c6ee304399dbdb9c8ef030ab642b10820\
                            db8f56000200000000000000000014",
                        "assetInIndex": 0,
                        "assetOutIndex": 1,
                        "amount": "1000000000000000000",
                        "userData": "0x",
                        "returnAmount": "227598784442065388110"
                    }
                ],
                "swapAmount": "1000000000000000000",
                "swapAmountForSwaps": "1000000000000000000",
                "returnAmount": "227598784442065388110",
                "returnAmountFromSwaps": "227598784442065388110",
                "returnAmountConsideringFees": "227307710853355710706",
                "tokenIn": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                "tokenOut": "0xba100000625a3754423978a60c9317c58a424e3d",
                "marketSp": "0.004393607339632106",
            }),
        },
    ])
    .await;

    let engine = tests::SolverEngine::new("balancer", balancer::config(&api.address)).await;

    let auction = json!({
        "id": null,
        "tokens": {
            "0xba100000625a3754423978a60c9317c58a424e3D": {
                "decimals": 18,
                "symbol": "BAL",
                "referencePrice": "4327903683155778",
                "availableBalance": "0",
                "trusted": true
            },
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2": {
                "decimals": 18,
                "symbol": "WETH",
                "referencePrice": "1000000000000000000",
                "availableBalance": "0",
                "trusted": true
            },
            "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee": {
                "decimals": 18,
                "symbol": "ETH",
                "referencePrice": "1000000000000000000",
                "availableBalance": "0",
                "trusted": true
            },
        },
        "orders": [
            {
                "uid": "0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a\
                          2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a\
                          2a2a2a2a",
                "sellToken": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                "buyToken": "0xba100000625a3754423978a60c9317c58a424e3D",
                "sellAmount": "16000000000000000000",
                "buyAmount": "3641580551073046209760",
                // Let's just assume 0 fee to not further complicate the math.
                "feeAmount": "0",
                "kind": "sell",
                "partiallyFillable": true,
                "class": "market",
                "reward": 0.
            }
        ],
        "liquidity": [],
        "effectiveGasPrice": "15000000000",
        "deadline": "2106-01-01T00:00:00.000Z"
    });

    for _ in 0..4 {
        let solution = engine.solve(auction.clone()).await;

        assert_eq!(
            solution,
            json!({
                "prices": {},
                "trades": [],
                "interactions": [],
            }),
        );
    }

    let solution = engine.solve(auction.clone()).await;

    // Solver finally found a solution after 5 tries.
    assert_eq!(
        solution,
        json!({
            "interactions": [
                {
                    "allowances": [
                        {
                            "amount": "1000000000000000000",
                            "spender": "0xba12222222228d8ba445958a75a0704d566bf2c8",
                            "token": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
                        }
                    ],
                    "calldata": "0x945bcec90000000000000000000000000000000000000000000\
                        00000000000000000000000000000000000000000000000000000000000000\
                        00000000000000000000120000000000000000000000000000000000000000\
                        00000000000000000000002200000000000000000000000009008d19f58aab\
                        d9ed0d60971565aa8510560ab4100000000000000000000000000000000000\
                        000000000000000000000000000000000000000000000000000009008d19f5\
                        8aabd9ed0d60971565aa8510560ab410000000000000000000000000000000\
                        00000000000000000000000000000000000000000000000000000000000000\
                        00000000000000000000000000000000280800000000000000000000000000\
                        00000000000000000000000000000000000000000000000000000000000000\
                        00000000000000000000000000000000000000100000000000000000000000\
                        000000000000000000000000000000000000000205c6ee304399dbdb9c8ef0\
                        30ab642b10820db8f560002000000000000000000140000000000000000000\
                        00000000000000000000000000000000000000000000000000000000000000\
                        00000000000000000000000000000000000000000000001000000000000000\
                        0000000000000000000000000000000000de0b6b3a76400000000000000000\
                        0000000000000000000000000000000000000000000000000a000000000000\
                        00000000000000000000000000000000000000000000000000000000000000\
                        00000000000000000000000000000000000000000000000000000020000000\
                        00000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc200000\
                        0000000000000000000ba100000625a3754423978a60c9317c58a424e3d000\
                        00000000000000000000000000000000000000000000000000000000000020\
                        000000000000000000000000000000000000000000000000de0b6b3a764000\
                        0fffffffffffffffffffffffffffffffffffffffffffffff3c9049e4e47ca5\
                        0ec",
                    "inputs": [
                        {
                            "amount": "1000000000000000000",
                            "token": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
                        }
                    ],
                    "internalize": false,
                    "kind": "custom",
                    "outputs": [
                        {
                            "amount": "227598784442065388110",
                            "token": "0xba100000625a3754423978a60c9317c58a424e3d"
                        }
                    ],
                    "target": "0xba12222222228d8ba445958a75a0704d566bf2c8",
                    "value": "0"
                }
            ],
            "prices": {
                "0xba100000625a3754423978a60c9317c58a424e3d": "1000000000000000000",
                "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2": "227598784442065388110"
            },
            "trades": [
                {
                    "executedAmount": "16000000000000000000",
                    "kind": "fulfillment",
                    "order": "0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a"
                }
            ]
        })
    );
}

/// Tests that we don't converge to 0 with the amounts we try to fill. Instead
/// we start over when our tried amount would be worth less than 0.01 ETH.
#[tokio::test]
async fn tested_amounts_wrap_around() {
    // Test is set up such that 2.5 BAL or exactly 0.01 ETH.
    // And the lowest amount we are willing to fill is 0.01 ETH.
    let fill_attempts = [
        "16000000000000000000", // 16 BAL == 0.064 ETH
        "8000000000000000000",  // 8  BAL == 0.032 ETH
        "4000000000000000000",  // 4  BAL == 0.016 ETH
        // Next would be 2 BAL == 0.008 ETH which is below
        // the minimum fill of 0.01 ETH so instead we start over.
        "16000000000000000000", // 16 BAL == 0.06 ETH
    ]
    .into_iter()
    .map(|amount| mock::http::Expectation::Post {
        path: mock::http::Path::Any,
        req: json!({
            "sellToken": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            "buyToken": "0xba100000625a3754423978a60c9317c58a424e3d",
            "orderKind": "buy",
            "amount": amount,
            "gasPrice": "15000000000",
        }),
        res: json!({
            "tokenAddresses": [],
            "swaps": [],
            "swapAmount": "0",
            "swapAmountForSwaps": "0",
            "returnAmount": "0",
            "returnAmountFromSwaps": "0",
            "returnAmountConsideringFees": "0",
            "tokenIn": "0x0000000000000000000000000000000000000000",
            "tokenOut": "0x0000000000000000000000000000000000000000",
            "marketSp": "0",
        }),
    })
    .collect();

    let api = mock::http::setup(fill_attempts).await;

    let engine = tests::SolverEngine::new("balancer", balancer::config(&api.address)).await;

    let auction = json!({
        "id": null,
        "tokens": {
            "0xba100000625a3754423978a60c9317c58a424e3D": {
                "decimals": 18,
                "symbol": "BAL",
                "referencePrice": "4000000000000000",
                "availableBalance": "0",
                "trusted": true
            },
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2": {
                "decimals": 18,
                "symbol": "WETH",
                "referencePrice": "1000000000000000000",
                "availableBalance": "0",
                "trusted": true
            },
            "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee": {
                "decimals": 18,
                "symbol": "ETH",
                "referencePrice": "1000000000000000000",
                "availableBalance": "0",
                "trusted": true
            },
        },
        "orders": [
            {
                "uid": "0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a\
                          2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a\
                          2a2a2a2a",
                "sellToken": "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
                "buyToken": "0xba100000625a3754423978a60c9317c58a424e3D",
                "sellAmount": "60000000000000000",
                "buyAmount": "16000000000000000000",
                // Let's just assume 0 fee to not further complicate the math.
                "feeAmount": "0",
                "kind": "buy",
                "partiallyFillable": true,
                "class": "market",
                "reward": 0.
            }
        ],
        "liquidity": [],
        "effectiveGasPrice": "15000000000",
        "deadline": "2106-01-01T00:00:00.000Z"
    });

    for _ in 0..4 {
        let solution = engine.solve(auction.clone()).await;
        assert_eq!(
            solution,
            json!({
                "prices": {},
                "trades": [],
                "interactions": [],
            }),
        );
    }
}