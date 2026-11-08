# Ejemplo de logs y eventos (localnet)

> Generado con `scripts/event-logs-example.ts` el 2026-11-08T18:55:12.887Z contra `solana-test-validator`. Pool: `Cza3rc5egzxeRbGscsuMqsAj2ieWpHXo4DwdpEVhih77`.

Cómo leerlo:

- **Logs:** cada transacción termina con `Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [2]`. Esa es la self-CPI del evento: el evento no aparece como texto en los logs.
- **Bytes de la self-CPI:** los datos de esa instrucción interna. Son 8 bytes de `EVENT_IX_TAG_LE` (`e445a52e51cb9a1d`), luego 8 bytes de discriminador del evento y después los campos en Borsh.
- **Evento decodificado:** lo que devuelve `program.coder.events.decode` (montos en unidades mínimas; el mint tiene 6 decimales).

Para regenerarlo: levantar `solana-test-validator` con el programa (o `anchor localnet`) y correr el comando del encabezado del script.

## initialize (fee 30 bps, con admin)

Firma: `3G3zCFJy6U9rddUFY6R2V2b4v63hGqVGHknL2RDuEhySm6U4JZUxndBozJrS2QYAZHaGnkHXFGtpoW7ZABXBCZG7` · compute units: 123146

### Logs del programa (`meta.logMessages`)

```text
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [1]
Program log: Instruction: Initialize
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program 11111111111111111111111111111111 invoke [2]
Program 11111111111111111111111111111111 success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: InitializeMint2
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 2780 of 181065 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]
Program log: Create
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: GetAccountDataSize
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1595 of 152095 compute units
Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program 11111111111111111111111111111111 invoke [3]
Program 11111111111111111111111111111111 success
Program log: Initialize the associated token account
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeImmutableOwner
Program log: Please upgrade to SPL Token 2022 for immutable owner support
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 145482 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeAccount3
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4214 of 141598 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 26490 of 163570 compute units
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]
Program log: Create
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: GetAccountDataSize
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1595 of 115068 compute units
Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program 11111111111111111111111111111111 invoke [3]
Program 11111111111111111111111111111111 success
Program log: Initialize the associated token account
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeImmutableOwner
Program log: Please upgrade to SPL Token 2022 for immutable owner support
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 108455 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeAccount3
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4214 of 104571 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 26490 of 126543 compute units
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [2]
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 3571 of 81545 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 123146 of 200000 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
```

### Bytes de la self-CPI (`meta.innerInstructions`, hex)

```text
e445a52e51cb9a1d6476ad570cc6fee50181c6e109f398047ee20ab425166c1336ab0f29a9483e244f879f50cff0b17947b23007fc00ac56eb73fe90b5ef6d686d4e47fbcb6781f36fa032b037899fee2a4506cd3c94d8fb20391a0f4e739d1c59e4dc4c53da7bb3d3a764168fe95a43cc859438be1b8ae2e3fb015edd59f333b00d19e1ff5920b2062c3f38620ee7bc6cbce9f49b485441633cf39f61e54516bfab2d8381b745a9ae8521e8de59c94b144cac83bca10100001e00
```

### Evento decodificado (`program.coder.events.decode`)

```json
[
  {
    "name": "poolInitialized",
    "data": {
      "authority": "9jbUzfSbDoc7hdzVD5iNDwnu9A9MMtKAkC4dQsbyPGm4",
      "config": "Cza3rc5egzxeRbGscsuMqsAj2ieWpHXo4DwdpEVhih77",
      "mintX": "5eT9WC5Jxj6gwq3WgLoumr1R28dRdBK2Rd1QrQLNkJhq",
      "mintY": "9zSK5qLQZSdzGhitx7RZgi4oPD5vMD2PzLKnHv8WggQs",
      "mintLp": "DiSZQ9RNMc154cd9zH9Cx5L1ThkwoaAjLPr4TffbUPns",
      "seed": "1794164108364",
      "fee": 30
    }
  }
]
```

## deposit (primer depósito 1:1, bloquea MINIMUM_LIQUIDITY)

Firma: `46uSFoigjC8t4ebUJn6DdCr7n7Ear3BFju95fNpgtuZ9RHDLQmEhu9xpf6687MCMN3KujeLqRkfct8GbiuHJaSDG` · compute units: 140607

### Logs del programa (`meta.logMessages`)

```text
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [1]
Program log: Instruction: Deposit
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]
Program log: Create
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: GetAccountDataSize
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1595 of 179537 compute units
Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program 11111111111111111111111111111111 invoke [3]
Program 11111111111111111111111111111111 success
Program log: Initialize the associated token account
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeImmutableOwner
Program log: Please upgrade to SPL Token 2022 for immutable owner support
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 172924 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeAccount3
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4214 of 169042 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 21889 of 186434 compute units
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [2]
Program log: Create
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: GetAccountDataSize
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1595 of 150231 compute units
Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program 11111111111111111111111111111111 invoke [3]
Program 11111111111111111111111111111111 success
Program log: Initialize the associated token account
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeImmutableOwner
Program log: Please upgrade to SPL Token 2022 for immutable owner support
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 1405 of 143618 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [3]
Program log: Instruction: InitializeAccount3
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4214 of 139734 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 20490 of 155706 compute units
Program ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: TransferChecked
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 6174 of 94179 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: TransferChecked
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 6174 of 85772 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: MintTo
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4492 of 77504 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: MintTo
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4492 of 70951 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [2]
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 3571 of 63778 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 140607 of 200000 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
```

### Bytes de la self-CPI (`meta.innerInstructions`, hex)

```text
e445a52e51cb9a1dda9b4ac13b425e7ab23007fc00ac56eb73fe90b5ef6d686d4e47fbcb6781f36fa032b037899fee2a81c6e109f398047ee20ab425166c1336ab0f29a9483e244f879f50cff0b1794740420f000000000040420f0000000000583e0f0000000000e80300000000000040420f000000000040420f0000000000
```

### Evento decodificado (`program.coder.events.decode`)

```json
[
  {
    "name": "liquidityDeposited",
    "data": {
      "config": "Cza3rc5egzxeRbGscsuMqsAj2ieWpHXo4DwdpEVhih77",
      "user": "9jbUzfSbDoc7hdzVD5iNDwnu9A9MMtKAkC4dQsbyPGm4",
      "amountX": "1000000",
      "amountY": "1000000",
      "lpMinted": "999000",
      "lpLocked": "1000",
      "reserveX": "1000000",
      "reserveY": "1000000"
    }
  }
]
```

## deposit desbalanceado (ofrece 100k X + 500k Y; solo se cobra la proporción)

Firma: `2UfLgi88aoM84Kgq1jFnunncQYWirAm8tWqriUKpyxLDmd2uuqPhGFv9xExaGa2UHMpzkehWywn4DSzxqRTDrMoX` · compute units: 77671

### Logs del programa (`meta.logMessages`)

```text
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [1]
Program log: Instruction: Deposit
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: TransferChecked
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 6174 of 150563 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: TransferChecked
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 6174 of 142156 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: MintTo
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4492 of 133888 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [2]
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 3571 of 126714 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 77671 of 200000 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
```

### Bytes de la self-CPI (`meta.innerInstructions`, hex)

```text
e445a52e51cb9a1dda9b4ac13b425e7ab23007fc00ac56eb73fe90b5ef6d686d4e47fbcb6781f36fa032b037899fee2a81c6e109f398047ee20ab425166c1336ab0f29a9483e244f879f50cff0b17947a086010000000000a086010000000000a0860100000000000000000000000000e0c8100000000000e0c8100000000000
```

### Evento decodificado (`program.coder.events.decode`)

```json
[
  {
    "name": "liquidityDeposited",
    "data": {
      "config": "Cza3rc5egzxeRbGscsuMqsAj2ieWpHXo4DwdpEVhih77",
      "user": "9jbUzfSbDoc7hdzVD5iNDwnu9A9MMtKAkC4dQsbyPGm4",
      "amountX": "100000",
      "amountY": "100000",
      "lpMinted": "100000",
      "lpLocked": "0",
      "reserveX": "1100000",
      "reserveY": "1100000"
    }
  }
]
```

## swap X→Y (10k de entrada)

Firma: `24T24qzbcrGR4SmF6DsQXL6aPUwUMhpaki9PT2oQZmQhzTpb5kJbnnFZv9212qrYnPkYajXr4M86dLBAV2bEVrVc` · compute units: 59735

### Logs del programa (`meta.logMessages`)

```text
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [1]
Program log: Instruction: Swap
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: TransferChecked
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 6174 of 161576 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: TransferChecked
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 6174 of 153167 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [2]
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 3571 of 144416 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 59735 of 200000 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
```

### Bytes de la self-CPI (`meta.innerInstructions`, hex)

```text
e445a52e51cb9a1d96a61ae11c59264fb23007fc00ac56eb73fe90b5ef6d686d4e47fbcb6781f36fa032b037899fee2a81c6e109f398047ee20ab425166c1336ab0f29a9483e244f879f50cff0b1794710270000000000009826000000000000f0ef10000000000048a210000000000001
```

### Evento decodificado (`program.coder.events.decode`)

```json
[
  {
    "name": "swapExecuted",
    "data": {
      "config": "Cza3rc5egzxeRbGscsuMqsAj2ieWpHXo4DwdpEVhih77",
      "user": "9jbUzfSbDoc7hdzVD5iNDwnu9A9MMtKAkC4dQsbyPGm4",
      "amountIn": "10000",
      "amountOut": "9880",
      "reserveX": "1110000",
      "reserveY": "1090120",
      "isX": true
    }
  }
]
```

## withdraw (quema 100k LP)

Firma: `5j9TMX4fCQ2Tr8khhgVL1F8deq2jmby6y1PgFesMWSe4wcvxijmgLgP7pfybw2KsnXUEmgUyBmdwML6uAKn8mvYS` · compute units: 72543

### Logs del programa (`meta.logMessages`)

```text
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [1]
Program log: Instruction: Withdraw
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: Burn
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4707 of 155884 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: TransferChecked
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 6174 of 148902 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]
Program log: Instruction: TransferChecked
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 6174 of 140492 compute units
Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [2]
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 3571 of 131701 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 72543 of 200000 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
```

### Bytes de la self-CPI (`meta.innerInstructions`, hex)

```text
e445a52e51cb9a1df078498b9a1fda44b23007fc00ac56eb73fe90b5ef6d686d4e47fbcb6781f36fa032b037899fee2a81c6e109f398047ee20ab425166c1336ab0f29a9483e244f879f50cff0b17947a0860100000000002d8a0100000000001d83010000000000c3650f00000000002b1f0f0000000000
```

### Evento decodificado (`program.coder.events.decode`)

```json
[
  {
    "name": "liquidityWithdrawn",
    "data": {
      "config": "Cza3rc5egzxeRbGscsuMqsAj2ieWpHXo4DwdpEVhih77",
      "user": "9jbUzfSbDoc7hdzVD5iNDwnu9A9MMtKAkC4dQsbyPGm4",
      "lpBurned": "100000",
      "amountX": "100909",
      "amountY": "99101",
      "reserveX": "1009091",
      "reserveY": "991019"
    }
  }
]
```

## set_locked(true)

Firma: `2VhBaWd11gNe4xRxCMZdBXjTwUo1bwcjaVYqQRt8q3AY5JSYU5R5PQeVesmgHjyncjKNPZ3BpbLw6upyDj6hyyZC` · compute units: 13159

### Logs del programa (`meta.logMessages`)

```text
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [1]
Program log: Instruction: SetLocked
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [2]
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 3571 of 191193 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 13159 of 200000 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
```

### Bytes de la self-CPI (`meta.innerInstructions`, hex)

```text
e445a52e51cb9a1df816b0d160269049b23007fc00ac56eb73fe90b5ef6d686d4e47fbcb6781f36fa032b037899fee2a81c6e109f398047ee20ab425166c1336ab0f29a9483e244f879f50cff0b1794701
```

### Evento decodificado (`program.coder.events.decode`)

```json
[
  {
    "name": "lockedUpdated",
    "data": {
      "config": "Cza3rc5egzxeRbGscsuMqsAj2ieWpHXo4DwdpEVhih77",
      "authority": "9jbUzfSbDoc7hdzVD5iNDwnu9A9MMtKAkC4dQsbyPGm4",
      "locked": true
    }
  }
]
```

## set_authority(None) — renuncia irreversible

Firma: `5SH5SgtmYTYvPSjHsmfxf4NFgWJ2wi23eV1fiBoFxFrMa4tL7C6KiPtejXaBhkdju41EkXUaNwa97QFkz5AFxqtU` · compute units: 13180

### Logs del programa (`meta.logMessages`)

```text
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [1]
Program log: Instruction: SetAuthority
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD invoke [2]
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 3571 of 191129 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD consumed 13180 of 200000 compute units
Program DR4UwHAVE9tVSm1kJo89ZiV6Dk1SXVPCPAhg67LT99mD success
```

### Bytes de la self-CPI (`meta.innerInstructions`, hex)

```text
e445a52e51cb9a1d85cf187a0eea5b220181c6e109f398047ee20ab425166c1336ab0f29a9483e244f879f50cff0b1794700b23007fc00ac56eb73fe90b5ef6d686d4e47fbcb6781f36fa032b037899fee2a
```

### Evento decodificado (`program.coder.events.decode`)

```json
[
  {
    "name": "authorityUpdated",
    "data": {
      "previousAuthority": "9jbUzfSbDoc7hdzVD5iNDwnu9A9MMtKAkC4dQsbyPGm4",
      "newAuthority": null,
      "config": "Cza3rc5egzxeRbGscsuMqsAj2ieWpHXo4DwdpEVhih77"
    }
  }
]
```
