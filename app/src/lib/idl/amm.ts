/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/amm.json`.
 */
export type Amm = {
  "address": "DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD",
  "metadata": {
    "name": "amm",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Constant-product AMM (x * y = k) on Solana with Anchor"
  },
  "docs": [
    "Program entrypoints for the AMM."
  ],
  "instructions": [
    {
      "name": "deposit",
      "docs": [
        "@notice Deposits X/Y liquidity and mints LP tokens to the user.",
        "@dev Locks `MINIMUM_LIQUIDITY` on the first deposit; enforces `min_lp` slippage.",
        "@param ctx Accounts context (`Deposit`).",
        "@param amount_x Amount of token X to deposit.",
        "@param amount_y Amount of token Y to deposit.",
        "@param min_lp Minimum acceptable LP minted (slippage protection).",
        "@return Result<()> Ok when the deposit completes."
      ],
      "discriminator": [
        242,
        35,
        198,
        137,
        82,
        225,
        242,
        182
      ],
      "accounts": [
        {
          "name": "user",
          "docs": [
            "Liquidity provider funding the deposit and paying ATA rent if needed."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "mintX",
          "docs": [
            "Mint of token X (must match `config.mint_x`)."
          ],
          "relations": [
            "config"
          ]
        },
        {
          "name": "mintY",
          "docs": [
            "Mint of token Y (must match `config.mint_y`)."
          ],
          "relations": [
            "config"
          ]
        },
        {
          "name": "mintLp",
          "docs": [
            "LP mint controlled by the Config PDA."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "config"
              }
            ]
          },
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
          "docs": [
            "Pool config PDA. Uses stored canonical bumps (no client-supplied bump)."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "config.seed",
                "account": "config"
              }
            ]
          }
        },
        {
          "name": "vaultX",
          "docs": [
            "Pool vault for token X."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintX"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "vaultY",
          "docs": [
            "Pool vault for token Y."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintY"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "userX",
          "docs": [
            "User ATA for token X."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintX"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "userY",
          "docs": [
            "User ATA for token Y."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintY"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "userLp",
          "docs": [
            "User ATA for LP tokens (created on first deposit if needed)."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintLp"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "lockLp",
          "docs": [
            "Locked LP ATA owned by Config (holds `MINIMUM_LIQUIDITY` forever)."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintLp"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL Token or Token-2022 program."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "docs": [
            "Associated Token Account program."
          ],
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "docs": [
            "System program."
          ],
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amountX",
          "type": "u64"
        },
        {
          "name": "amountY",
          "type": "u64"
        },
        {
          "name": "minLp",
          "type": "u64"
        }
      ]
    },
    {
      "name": "initialize",
      "docs": [
        "@notice Initializes a new constant-product pool.",
        "@dev Creates Config PDA, LP mint, and vault ATAs; stores canonical bumps.",
        "@param ctx Accounts context (`Initialize`).",
        "@param seed PDA seed for Config.",
        "@param fee Swap fee in basis points.",
        "@param authority Optional admin; `None` for immutable / no admin.",
        "@return Result<()> Ok when the pool accounts are initialized."
      ],
      "discriminator": [
        175,
        175,
        109,
        31,
        13,
        152,
        155,
        237
      ],
      "accounts": [
        {
          "name": "initializer",
          "docs": [
            "Payer and transaction signer that funds account creation."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "mintX",
          "docs": [
            "Mint of token X in the trading pair."
          ]
        },
        {
          "name": "mintY",
          "docs": [
            "Mint of token Y in the trading pair."
          ]
        },
        {
          "name": "config",
          "docs": [
            "Pool configuration PDA. Space is exactly `Config::ACCOUNT_SPACE`."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              },
              {
                "kind": "arg",
                "path": "seed"
              }
            ]
          }
        },
        {
          "name": "mintLp",
          "docs": [
            "LP share mint. Mint authority is the Config PDA."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "config"
              }
            ]
          }
        },
        {
          "name": "vaultX",
          "docs": [
            "Vault holding reserves of token X. Created in the handler via ATA CPI."
          ],
          "writable": true
        },
        {
          "name": "vaultY",
          "docs": [
            "Vault holding reserves of token Y. Created in the handler via ATA CPI."
          ],
          "writable": true
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL Token or Token-2022 program (validated Interface — anti arbitrary CPI)."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "docs": [
            "Associated Token Account program."
          ],
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "docs": [
            "System program for account creation."
          ],
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "seed",
          "type": "u64"
        },
        {
          "name": "fee",
          "type": "u16"
        },
        {
          "name": "authority",
          "type": {
            "option": "pubkey"
          }
        }
      ]
    },
    {
      "name": "swap",
      "docs": [
        "@notice Swaps an exact input amount for the other pool token.",
        "@dev Fee on input; enforces constant-product invariant and `min_amount_out`.",
        "@param ctx Accounts context (`Swap`).",
        "@param is_x `true` for X→Y, `false` for Y→X.",
        "@param amount_in Exact tokens sent by the user.",
        "@param min_amount_out Minimum acceptable output (slippage).",
        "@return Result<()> Ok when both transfers complete."
      ],
      "discriminator": [
        248,
        198,
        158,
        145,
        225,
        117,
        135,
        200
      ],
      "accounts": [
        {
          "name": "user",
          "docs": [
            "Trader funding the input transfer and receiving the output."
          ],
          "signer": true
        },
        {
          "name": "mintX",
          "docs": [
            "Mint of token X (must match `config.mint_x`)."
          ],
          "relations": [
            "config"
          ]
        },
        {
          "name": "mintY",
          "docs": [
            "Mint of token Y (must match `config.mint_y`)."
          ],
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
          "docs": [
            "Pool config PDA. Uses stored canonical bumps (no client-supplied bump)."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "config.seed",
                "account": "config"
              }
            ]
          }
        },
        {
          "name": "vaultX",
          "docs": [
            "Pool vault for token X."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintX"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "vaultY",
          "docs": [
            "Pool vault for token Y."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintY"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "userX",
          "docs": [
            "User ATA for token X."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintX"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "userY",
          "docs": [
            "User ATA for token Y."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintY"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL Token or Token-2022 program."
          ]
        }
      ],
      "args": [
        {
          "name": "isX",
          "type": "bool"
        },
        {
          "name": "amountIn",
          "type": "u64"
        },
        {
          "name": "minAmountOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "withdraw",
      "docs": [
        "@notice Burns LP and withdraws proportional X/Y to the user.",
        "@dev Floor division favors the pool; cannot burn below `MINIMUM_LIQUIDITY` supply.",
        "@param ctx Accounts context (`Withdraw`).",
        "@param lp_amount LP tokens to burn.",
        "@param min_x Minimum token X out (slippage).",
        "@param min_y Minimum token Y out (slippage).",
        "@return Result<()> Ok when burn and transfers complete."
      ],
      "discriminator": [
        183,
        18,
        70,
        156,
        148,
        109,
        161,
        34
      ],
      "accounts": [
        {
          "name": "user",
          "docs": [
            "Liquidity provider burning LP and receiving X/Y."
          ],
          "signer": true
        },
        {
          "name": "mintX",
          "docs": [
            "Mint of token X (must match `config.mint_x`)."
          ],
          "relations": [
            "config"
          ]
        },
        {
          "name": "mintY",
          "docs": [
            "Mint of token Y (must match `config.mint_y`)."
          ],
          "relations": [
            "config"
          ]
        },
        {
          "name": "mintLp",
          "docs": [
            "LP mint controlled by the Config PDA."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  112
                ]
              },
              {
                "kind": "account",
                "path": "config"
              }
            ]
          },
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
          "docs": [
            "Pool config PDA. Uses stored canonical bumps (no client-supplied bump)."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "config.seed",
                "account": "config"
              }
            ]
          }
        },
        {
          "name": "vaultX",
          "docs": [
            "Pool vault for token X."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintX"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "vaultY",
          "docs": [
            "Pool vault for token Y."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintY"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "userX",
          "docs": [
            "User ATA for token X."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintX"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "userY",
          "docs": [
            "User ATA for token Y."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintY"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "userLp",
          "docs": [
            "User ATA holding LP tokens to burn."
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "mintLp"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL Token or Token-2022 program."
          ]
        }
      ],
      "args": [
        {
          "name": "lpAmount",
          "type": "u64"
        },
        {
          "name": "minX",
          "type": "u64"
        },
        {
          "name": "minY",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "config",
      "discriminator": [
        155,
        12,
        170,
        224,
        30,
        250,
        204,
        130
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "identicalVaults",
      "msg": "Vault or token accounts for X and Y must not be identical"
    },
    {
      "code": 6001,
      "name": "identicalMints",
      "msg": "Mint X and mint Y must not be identical"
    },
    {
      "code": 6002,
      "name": "invalidFee",
      "msg": "Fee is invalid (must be <= 10000 bps)"
    },
    {
      "code": 6003,
      "name": "slippageExceeded",
      "msg": "Slippage tolerance exceeded"
    },
    {
      "code": 6004,
      "name": "mathOverflow",
      "msg": "Arithmetic overflow or invalid math"
    },
    {
      "code": 6005,
      "name": "insufficientLiquidity",
      "msg": "Insufficient liquidity"
    },
    {
      "code": 6006,
      "name": "unauthorized",
      "msg": "unauthorized"
    },
    {
      "code": 6007,
      "name": "poolLocked",
      "msg": "Pool is locked"
    },
    {
      "code": 6008,
      "name": "invalidAmount",
      "msg": "Invalid amount"
    }
  ],
  "types": [
    {
      "name": "config",
      "docs": [
        "On-chain configuration for a constant-product AMM pool (`x * y = k`).",
        "",
        "Allocated with exact space `8 (discriminator) + Config::INIT_SPACE`.",
        "Canonical bumps are persisted here so later instructions use",
        "`bump = config.config_bump` instead of client-supplied bumps."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "docs": [
              "Optional protocol / admin authority. `None` means immutable / no admin."
            ],
            "type": {
              "option": "pubkey"
            }
          },
          {
            "name": "mintX",
            "docs": [
              "Mint of token X in the pair."
            ],
            "type": "pubkey"
          },
          {
            "name": "mintY",
            "docs": [
              "Mint of token Y in the pair."
            ],
            "type": "pubkey"
          },
          {
            "name": "mintLp",
            "docs": [
              "LP share mint whose mint authority is the Config PDA."
            ],
            "type": "pubkey"
          },
          {
            "name": "seed",
            "docs": [
              "Client-chosen seed used in the Config PDA derivation."
            ],
            "type": "u64"
          },
          {
            "name": "fee",
            "docs": [
              "Swap fee in basis points (1 bps = 0.01%). Must be `<= MAX_FEE_BPS`."
            ],
            "type": "u16"
          },
          {
            "name": "locked",
            "docs": [
              "When `true`, deposits / swaps / withdraws that mutate pool state are rejected."
            ],
            "type": "bool"
          },
          {
            "name": "configBump",
            "docs": [
              "Canonical bump for the Config PDA."
            ],
            "type": "u8"
          },
          {
            "name": "lpBump",
            "docs": [
              "Canonical bump for the LP mint PDA."
            ],
            "type": "u8"
          }
        ]
      }
    }
  ]
};
