/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/advisor_fee_vault.json`.
 */
export type AdvisorFeeVault = {
  "address": "F4Q5gatG2GjEt8L62Q71z7jctaitHRxunJDDnbdo6oXm",
  "metadata": {
    "name": "advisorFeeVault",
    "version": "0.1.0",
    "spec": "0.1.0"
  },
  "instructions": [
    {
      "name": "accrue",
      "discriminator": [
        23,
        76,
        128,
        149,
        229,
        247,
        72,
        228
      ],
      "accounts": [
        {
          "name": "vault",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "vault.config",
                "account": "vault"
              },
              {
                "kind": "account",
                "path": "source"
              }
            ]
          },
          "relations": [
            "position"
          ]
        },
        {
          "name": "source",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              },
              {
                "kind": "account",
                "path": "position.client",
                "account": "position"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "closePosition",
      "discriminator": [
        123,
        134,
        81,
        0,
        49,
        68,
        98,
        98
      ],
      "accounts": [
        {
          "name": "client",
          "writable": true,
          "signer": true,
          "relations": [
            "position"
          ]
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "position.vault",
                "account": "position"
              },
              {
                "kind": "account",
                "path": "client"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "deposit",
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
          "name": "client",
          "signer": true
        },
        {
          "name": "clientAssets",
          "writable": true
        },
        {
          "name": "settlement",
          "accounts": [
            {
              "name": "config",
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
                  }
                ]
              },
              "relations": [
                "vault"
              ]
            },
            {
              "name": "vault",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      118,
                      97,
                      117,
                      108,
                      116
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "config"
                  },
                  {
                    "kind": "account",
                    "path": "source"
                  }
                ]
              },
              "relations": [
                "position"
              ]
            },
            {
              "name": "position",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      112,
                      111,
                      115,
                      105,
                      116,
                      105,
                      111,
                      110
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "vault"
                  },
                  {
                    "kind": "account",
                    "path": "position.client",
                    "account": "position"
                  }
                ]
              }
            },
            {
              "name": "source",
              "writable": true,
              "relations": [
                "vault"
              ]
            },
            {
              "name": "mint",
              "relations": [
                "config",
                "source"
              ]
            },
            {
              "name": "sharesMint",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      115,
                      104,
                      97,
                      114,
                      101,
                      115
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "source"
                  }
                ],
                "program": {
                  "kind": "const",
                  "value": [
                    59,
                    41,
                    166,
                    132,
                    113,
                    223,
                    230,
                    190,
                    131,
                    243,
                    50,
                    164,
                    161,
                    73,
                    14,
                    118,
                    239,
                    139,
                    8,
                    202,
                    64,
                    185,
                    149,
                    143,
                    65,
                    50,
                    106,
                    235,
                    128,
                    128,
                    56,
                    238
                  ]
                }
              }
            },
            {
              "name": "reserve",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      114,
                      101,
                      115,
                      101,
                      114,
                      118,
                      101
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "source"
                  }
                ],
                "program": {
                  "kind": "const",
                  "value": [
                    59,
                    41,
                    166,
                    132,
                    113,
                    223,
                    230,
                    190,
                    131,
                    243,
                    50,
                    164,
                    161,
                    73,
                    14,
                    118,
                    239,
                    139,
                    8,
                    202,
                    64,
                    185,
                    149,
                    143,
                    65,
                    50,
                    106,
                    235,
                    128,
                    128,
                    56,
                    238
                  ]
                }
              }
            },
            {
              "name": "cash",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      99,
                      97,
                      115,
                      104
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "vault"
                  }
                ]
              }
            },
            {
              "name": "custody",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      99,
                      117,
                      115,
                      116,
                      111,
                      100,
                      121
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "vault"
                  }
                ]
              }
            },
            {
              "name": "officeAssets",
              "writable": true
            },
            {
              "name": "advisorAssets",
              "writable": true
            },
            {
              "name": "rivoolAssets",
              "writable": true
            },
            {
              "name": "sourceProgram",
              "address": "4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh"
            },
            {
              "name": "tokenProgram",
              "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            }
          ]
        }
      ],
      "args": [
        {
          "name": "assets",
          "type": "u64"
        },
        {
          "name": "minShares",
          "type": "u64"
        }
      ]
    },
    {
      "name": "initializeConfig",
      "discriminator": [
        208,
        127,
        21,
        1,
        194,
        190,
        196,
        70
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "program",
          "address": "F4Q5gatG2GjEt8L62Q71z7jctaitHRxunJDDnbdo6oXm"
        },
        {
          "name": "programData"
        },
        {
          "name": "mint"
        },
        {
          "name": "config",
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
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "treasury",
          "type": "pubkey"
        },
        {
          "name": "demo",
          "type": "bool"
        }
      ]
    },
    {
      "name": "openPosition",
      "discriminator": [
        135,
        128,
        47,
        77,
        15,
        152,
        240,
        49
      ],
      "accounts": [
        {
          "name": "client",
          "writable": true,
          "signer": true
        },
        {
          "name": "config",
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
              }
            ]
          },
          "relations": [
            "vault",
            "office"
          ]
        },
        {
          "name": "vault",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "source"
              }
            ]
          }
        },
        {
          "name": "source",
          "relations": [
            "vault"
          ]
        },
        {
          "name": "office",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  105,
                  99,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "office.initialAuthority",
                "account": "office"
              }
            ]
          },
          "relations": [
            "advisor"
          ]
        },
        {
          "name": "advisor",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  118,
                  105,
                  115,
                  111,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "office"
              },
              {
                "kind": "account",
                "path": "advisor.authority",
                "account": "advisor"
              }
            ]
          }
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              },
              {
                "kind": "account",
                "path": "client"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "rates",
          "type": {
            "array": [
              "u16",
              2
            ]
          }
        },
        {
          "name": "recipients",
          "type": {
            "array": [
              "pubkey",
              3
            ]
          }
        }
      ]
    },
    {
      "name": "registerAdvisor",
      "discriminator": [
        66,
        35,
        33,
        184,
        184,
        82,
        248,
        80
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true,
          "relations": [
            "office"
          ]
        },
        {
          "name": "office",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  105,
                  99,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "office.config",
                "account": "office"
              },
              {
                "kind": "account",
                "path": "office.initialAuthority",
                "account": "office"
              }
            ]
          }
        },
        {
          "name": "advisorAuthority"
        },
        {
          "name": "advisor",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  118,
                  105,
                  115,
                  111,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "office"
              },
              {
                "kind": "account",
                "path": "advisorAuthority"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "treasury",
          "type": "pubkey"
        },
        {
          "name": "feeBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "registerOffice",
      "discriminator": [
        192,
        207,
        155,
        152,
        133,
        154,
        119,
        132
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true,
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
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
              }
            ]
          }
        },
        {
          "name": "officeAuthority"
        },
        {
          "name": "office",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  105,
                  99,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "officeAuthority"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "treasury",
          "type": "pubkey"
        },
        {
          "name": "feeBps",
          "type": "u16"
        },
        {
          "name": "advisorAllotmentBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "registerVault",
      "discriminator": [
        121,
        62,
        4,
        122,
        93,
        231,
        119,
        49
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true,
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
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
              }
            ]
          }
        },
        {
          "name": "mint",
          "relations": [
            "config",
            "source"
          ]
        },
        {
          "name": "source"
        },
        {
          "name": "sharesMint",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  104,
                  97,
                  114,
                  101,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "source"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                59,
                41,
                166,
                132,
                113,
                223,
                230,
                190,
                131,
                243,
                50,
                164,
                161,
                73,
                14,
                118,
                239,
                139,
                8,
                202,
                64,
                185,
                149,
                143,
                65,
                50,
                106,
                235,
                128,
                128,
                56,
                238
              ]
            }
          }
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "source"
              }
            ]
          }
        },
        {
          "name": "cash",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  97,
                  115,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              }
            ]
          }
        },
        {
          "name": "custody",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  117,
                  115,
                  116,
                  111,
                  100,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "setAdvisorFee",
      "discriminator": [
        211,
        135,
        37,
        115,
        72,
        145,
        152,
        201
      ],
      "accounts": [
        {
          "name": "authority",
          "signer": true,
          "relations": [
            "advisor"
          ]
        },
        {
          "name": "office",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  105,
                  99,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "office.config",
                "account": "office"
              },
              {
                "kind": "account",
                "path": "office.initialAuthority",
                "account": "office"
              }
            ]
          },
          "relations": [
            "advisor"
          ]
        },
        {
          "name": "advisor",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  118,
                  105,
                  115,
                  111,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "office"
              },
              {
                "kind": "account",
                "path": "authority"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "feeBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "setOfficeFee",
      "discriminator": [
        158,
        191,
        207,
        204,
        146,
        51,
        255,
        30
      ],
      "accounts": [
        {
          "name": "authority",
          "signer": true,
          "relations": [
            "office"
          ]
        },
        {
          "name": "office",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  105,
                  99,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "office.config",
                "account": "office"
              },
              {
                "kind": "account",
                "path": "office.initialAuthority",
                "account": "office"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "feeBps",
          "type": "u16"
        },
        {
          "name": "advisorAllotmentBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "settle",
      "discriminator": [
        175,
        42,
        185,
        87,
        144,
        131,
        102,
        212
      ],
      "accounts": [
        {
          "name": "config",
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
              }
            ]
          },
          "relations": [
            "vault"
          ]
        },
        {
          "name": "vault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "config"
              },
              {
                "kind": "account",
                "path": "source"
              }
            ]
          },
          "relations": [
            "position"
          ]
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              },
              {
                "kind": "account",
                "path": "position.client",
                "account": "position"
              }
            ]
          }
        },
        {
          "name": "source",
          "writable": true,
          "relations": [
            "vault"
          ]
        },
        {
          "name": "mint",
          "relations": [
            "config",
            "source"
          ]
        },
        {
          "name": "sharesMint",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  115,
                  104,
                  97,
                  114,
                  101,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "source"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                59,
                41,
                166,
                132,
                113,
                223,
                230,
                190,
                131,
                243,
                50,
                164,
                161,
                73,
                14,
                118,
                239,
                139,
                8,
                202,
                64,
                185,
                149,
                143,
                65,
                50,
                106,
                235,
                128,
                128,
                56,
                238
              ]
            }
          }
        },
        {
          "name": "reserve",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  114,
                  101,
                  115,
                  101,
                  114,
                  118,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "source"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                59,
                41,
                166,
                132,
                113,
                223,
                230,
                190,
                131,
                243,
                50,
                164,
                161,
                73,
                14,
                118,
                239,
                139,
                8,
                202,
                64,
                185,
                149,
                143,
                65,
                50,
                106,
                235,
                128,
                128,
                56,
                238
              ]
            }
          }
        },
        {
          "name": "cash",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  97,
                  115,
                  104
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              }
            ]
          }
        },
        {
          "name": "custody",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  117,
                  115,
                  116,
                  111,
                  100,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "vault"
              }
            ]
          }
        },
        {
          "name": "officeAssets",
          "writable": true
        },
        {
          "name": "advisorAssets",
          "writable": true
        },
        {
          "name": "rivoolAssets",
          "writable": true
        },
        {
          "name": "sourceProgram",
          "address": "4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": []
    },
    {
      "name": "transferConfigAuthority",
      "discriminator": [
        105,
        153,
        105,
        110,
        203,
        141,
        163,
        235
      ],
      "accounts": [
        {
          "name": "authority",
          "signer": true,
          "relations": [
            "config"
          ]
        },
        {
          "name": "config",
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
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "authority",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "transferOfficeAuthority",
      "discriminator": [
        62,
        158,
        117,
        92,
        154,
        180,
        108,
        251
      ],
      "accounts": [
        {
          "name": "authority",
          "signer": true,
          "relations": [
            "office"
          ]
        },
        {
          "name": "office",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  105,
                  99,
                  101
                ]
              },
              {
                "kind": "account",
                "path": "office.config",
                "account": "office"
              },
              {
                "kind": "account",
                "path": "office.initialAuthority",
                "account": "office"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "authority",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "withdraw",
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
          "name": "client",
          "signer": true
        },
        {
          "name": "clientAssets",
          "writable": true
        },
        {
          "name": "settlement",
          "accounts": [
            {
              "name": "config",
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
                  }
                ]
              },
              "relations": [
                "vault"
              ]
            },
            {
              "name": "vault",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      118,
                      97,
                      117,
                      108,
                      116
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "config"
                  },
                  {
                    "kind": "account",
                    "path": "source"
                  }
                ]
              },
              "relations": [
                "position"
              ]
            },
            {
              "name": "position",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      112,
                      111,
                      115,
                      105,
                      116,
                      105,
                      111,
                      110
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "vault"
                  },
                  {
                    "kind": "account",
                    "path": "position.client",
                    "account": "position"
                  }
                ]
              }
            },
            {
              "name": "source",
              "writable": true,
              "relations": [
                "vault"
              ]
            },
            {
              "name": "mint",
              "relations": [
                "config",
                "source"
              ]
            },
            {
              "name": "sharesMint",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      115,
                      104,
                      97,
                      114,
                      101,
                      115
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "source"
                  }
                ],
                "program": {
                  "kind": "const",
                  "value": [
                    59,
                    41,
                    166,
                    132,
                    113,
                    223,
                    230,
                    190,
                    131,
                    243,
                    50,
                    164,
                    161,
                    73,
                    14,
                    118,
                    239,
                    139,
                    8,
                    202,
                    64,
                    185,
                    149,
                    143,
                    65,
                    50,
                    106,
                    235,
                    128,
                    128,
                    56,
                    238
                  ]
                }
              }
            },
            {
              "name": "reserve",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      114,
                      101,
                      115,
                      101,
                      114,
                      118,
                      101
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "source"
                  }
                ],
                "program": {
                  "kind": "const",
                  "value": [
                    59,
                    41,
                    166,
                    132,
                    113,
                    223,
                    230,
                    190,
                    131,
                    243,
                    50,
                    164,
                    161,
                    73,
                    14,
                    118,
                    239,
                    139,
                    8,
                    202,
                    64,
                    185,
                    149,
                    143,
                    65,
                    50,
                    106,
                    235,
                    128,
                    128,
                    56,
                    238
                  ]
                }
              }
            },
            {
              "name": "cash",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      99,
                      97,
                      115,
                      104
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "vault"
                  }
                ]
              }
            },
            {
              "name": "custody",
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      99,
                      117,
                      115,
                      116,
                      111,
                      100,
                      121
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "vault"
                  }
                ]
              }
            },
            {
              "name": "officeAssets",
              "writable": true
            },
            {
              "name": "advisorAssets",
              "writable": true
            },
            {
              "name": "rivoolAssets",
              "writable": true
            },
            {
              "name": "sourceProgram",
              "address": "4ywtd7N6oR52qPRiZ66mX76HPpjHAXiCYqoyaNVFF1xh"
            },
            {
              "name": "tokenProgram",
              "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            }
          ]
        }
      ],
      "args": [
        {
          "name": "shares",
          "type": {
            "option": "u64"
          }
        },
        {
          "name": "minAssets",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "advisor",
      "discriminator": [
        137,
        159,
        140,
        110,
        221,
        88,
        111,
        60
      ]
    },
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
    },
    {
      "name": "office",
      "discriminator": [
        59,
        52,
        41,
        106,
        16,
        181,
        109,
        205
      ]
    },
    {
      "name": "position",
      "discriminator": [
        170,
        188,
        143,
        228,
        122,
        64,
        247,
        208
      ]
    },
    {
      "name": "vault",
      "discriminator": [
        211,
        8,
        232,
        43,
        2,
        152,
        117,
        119
      ]
    }
  ],
  "events": [
    {
      "name": "advisorFeeSet",
      "discriminator": [
        173,
        80,
        9,
        118,
        80,
        28,
        76,
        115
      ]
    },
    {
      "name": "authorityTransferred",
      "discriminator": [
        245,
        109,
        179,
        54,
        135,
        92,
        22,
        64
      ]
    },
    {
      "name": "configCreated",
      "discriminator": [
        195,
        73,
        104,
        161,
        166,
        245,
        4,
        120
      ]
    },
    {
      "name": "feesAccrued",
      "discriminator": [
        1,
        151,
        46,
        93,
        244,
        90,
        12,
        191
      ]
    },
    {
      "name": "feesSettled",
      "discriminator": [
        67,
        117,
        237,
        13,
        182,
        185,
        211,
        44
      ]
    },
    {
      "name": "officeFeeSet",
      "discriminator": [
        87,
        33,
        98,
        181,
        209,
        251,
        175,
        132
      ]
    },
    {
      "name": "positionDeposited",
      "discriminator": [
        198,
        104,
        233,
        60,
        18,
        93,
        100,
        50
      ]
    },
    {
      "name": "positionWithdrawn",
      "discriminator": [
        207,
        105,
        38,
        76,
        190,
        32,
        8,
        81
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "positionFunded",
      "msg": "Position still owns assets"
    },
    {
      "code": 6001,
      "name": "termsChanged",
      "msg": "Accepted terms changed before signing"
    },
    {
      "code": 6002,
      "name": "notDue",
      "msg": "Settlement interval has not elapsed"
    },
    {
      "code": 6003,
      "name": "invalidAmount",
      "msg": "Amount must be positive and within position balance"
    },
    {
      "code": 6004,
      "name": "slippage",
      "msg": "Minimum output not met"
    },
    {
      "code": 6005,
      "name": "invalidCheckpoint",
      "msg": "Clock or price index moved backwards"
    },
    {
      "code": 6006,
      "name": "feeCeiling",
      "msg": "Office fee and advisor allotment must fit within 100 basis points"
    },
    {
      "code": 6007,
      "name": "advisorAllotment",
      "msg": "Advisor fee exceeds the office allotment"
    },
    {
      "code": 6008,
      "name": "overflow",
      "msg": "Arithmetic overflow"
    },
    {
      "code": 6009,
      "name": "invalidMathInput",
      "msg": "Invalid fixed-point input or remainder"
    },
    {
      "code": 6010,
      "name": "unauthorized",
      "msg": "Required authority did not sign"
    },
    {
      "code": 6011,
      "name": "invalidMint",
      "msg": "Only six-decimal SPL token mints are supported"
    },
    {
      "code": 6012,
      "name": "invalidTreasury",
      "msg": "Treasury must be a nonzero public key"
    }
  ],
  "types": [
    {
      "name": "advisor",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "office",
            "type": "pubkey"
          },
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "treasury",
            "type": "pubkey"
          },
          {
            "name": "feeBps",
            "type": "u16"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "advisorFeeSet",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "advisor",
            "type": "pubkey"
          },
          {
            "name": "feeBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "authorityTransferred",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "account",
            "type": "pubkey"
          },
          {
            "name": "authority",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "config",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "treasury",
            "type": "pubkey"
          },
          {
            "name": "settlementInterval",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "configCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "config",
            "type": "pubkey"
          },
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "demo",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "feesAccrued",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "owed",
            "type": {
              "array": [
                "u64",
                2
              ]
            }
          },
          {
            "name": "at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "feesSettled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "grossAssets",
            "type": "u64"
          },
          {
            "name": "paid",
            "type": {
              "array": [
                "u64",
                3
              ]
            }
          },
          {
            "name": "at",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "office",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "config",
            "type": "pubkey"
          },
          {
            "name": "initialAuthority",
            "type": "pubkey"
          },
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "treasury",
            "type": "pubkey"
          },
          {
            "name": "feeBps",
            "type": "u16"
          },
          {
            "name": "advisorAllotmentBps",
            "type": "u16"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "officeFeeSet",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "office",
            "type": "pubkey"
          },
          {
            "name": "feeBps",
            "type": "u16"
          },
          {
            "name": "advisorAllotmentBps",
            "type": "u16"
          }
        ]
      }
    },
    {
      "name": "position",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "client",
            "type": "pubkey"
          },
          {
            "name": "vault",
            "type": "pubkey"
          },
          {
            "name": "office",
            "type": "pubkey"
          },
          {
            "name": "advisor",
            "type": "pubkey"
          },
          {
            "name": "rates",
            "type": {
              "array": [
                "u16",
                2
              ]
            }
          },
          {
            "name": "recipients",
            "type": {
              "array": [
                "pubkey",
                3
              ]
            }
          },
          {
            "name": "shares",
            "type": "u64"
          },
          {
            "name": "cash",
            "type": "u64"
          },
          {
            "name": "highWaterMark",
            "type": "u128"
          },
          {
            "name": "priceIndex",
            "type": "u128"
          },
          {
            "name": "accruedAt",
            "type": "i64"
          },
          {
            "name": "settledAt",
            "type": "i64"
          },
          {
            "name": "owed",
            "type": {
              "array": [
                "u64",
                2
              ]
            }
          },
          {
            "name": "remainders",
            "type": {
              "array": [
                "u128",
                3
              ]
            }
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "positionDeposited",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "assets",
            "type": "u64"
          },
          {
            "name": "shares",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "positionWithdrawn",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "assets",
            "type": "u64"
          },
          {
            "name": "shares",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "vault",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "config",
            "type": "pubkey"
          },
          {
            "name": "source",
            "type": "pubkey"
          },
          {
            "name": "totalShares",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "yieldVault",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          },
          {
            "name": "mint",
            "type": "pubkey"
          },
          {
            "name": "price",
            "type": "u128"
          },
          {
            "name": "priceTimeIndex",
            "type": "u128"
          },
          {
            "name": "updatedAt",
            "type": "i64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    }
  ]
};
