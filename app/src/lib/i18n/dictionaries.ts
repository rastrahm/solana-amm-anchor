export type Locale = "en" | "es";

export type Dictionary = {
  brandCluster: (cluster: string) => string;
  themeToLight: string;
  themeToDark: string;
  themeLightLabel: string;
  themeDarkLabel: string;
  localeToEs: string;
  localeToEn: string;
  helpOpen: string;
  helpClose: string;
  helpTitle: string;
  heroTitle: string;
  heroBody: string;
  mintsTitle: string;
  mintsHint: string;
  mintX: string;
  mintY: string;
  initializeTitle: string;
  depositTitle: string;
  swapTitle: string;
  withdrawTitle: string;
  seed: string;
  feeBps: string;
  amountX: string;
  amountY: string;
  minLp: string;
  direction: string;
  swapXy: string;
  swapYx: string;
  amountIn: string;
  minOut: string;
  lpAmount: string;
  minX: string;
  minY: string;
  actionInitialize: string;
  actionDeposit: string;
  actionSwap: string;
  actionWithdraw: string;
  connectWalletFirst: string;
  invalidInput: string;
  initializeFailed: string;
  depositFailed: string;
  swapFailed: string;
  withdrawFailed: string;
  initialized: (config: string, txPrefix: string) => string;
  deposited: (txPrefix: string) => string;
  swapped: (txPrefix: string) => string;
  withdrawn: (txPrefix: string) => string;
  errorTitle: string;
  errorRetry: string;
  notFoundTitle: string;
  notFoundBody: string;
  notFoundHome: string;
  help: {
    intro: string;
    stepsTitle: string;
    steps: string[];
    fieldsTitle: string;
    fields: string[];
    phantomTitle: string;
    phantom: string[];
    tipsTitle: string;
    tips: string[];
  };
};

export const dictionaries: Record<Locale, Dictionary> = {
  en: {
    brandCluster: (cluster) => `cluster ${cluster}`,
    themeToLight: "Switch to light mode",
    themeToDark: "Switch to dark mode",
    themeLightLabel: "Light",
    themeDarkLabel: "Dark",
    localeToEs: "Cambiar a español",
    localeToEn: "Switch to English",
    helpOpen: "Open help",
    helpClose: "Close help",
    helpTitle: "How to use this AMM demo",
    heroTitle: "Trade and provide liquidity on your local pool",
    heroBody:
      "Connect a wallet, initialize a constant-product pool, then deposit, swap, or withdraw. Point NEXT_PUBLIC_SOLANA_RPC_URL at localnet or devnet.",
    mintsTitle: "Active mints",
    mintsHint:
      "Paste SPL mint addresses used by your pool. Create them with the CLI on localnet before depositing or swapping.",
    mintX: "Mint X",
    mintY: "Mint Y",
    initializeTitle: "Initialize pool",
    depositTitle: "Deposit",
    swapTitle: "Swap",
    withdrawTitle: "Withdraw",
    seed: "Seed",
    feeBps: "Fee (bps)",
    amountX: "Amount X",
    amountY: "Amount Y",
    minLp: "Min LP",
    direction: "Direction",
    swapXy: "X → Y",
    swapYx: "Y → X",
    amountIn: "Amount in",
    minOut: "Min out",
    lpAmount: "LP amount",
    minX: "Min X",
    minY: "Min Y",
    actionInitialize: "Initialize",
    actionDeposit: "Deposit",
    actionSwap: "Swap",
    actionWithdraw: "Withdraw",
    connectWalletFirst: "Connect a wallet first",
    invalidInput: "Invalid input",
    initializeFailed: "Initialize failed",
    depositFailed: "Deposit failed",
    swapFailed: "Swap failed",
    withdrawFailed: "Withdraw failed",
    initialized: (config, txPrefix) =>
      `Initialized. Config ${config} · tx ${txPrefix}…`,
    deposited: (txPrefix) => `Deposited · tx ${txPrefix}…`,
    swapped: (txPrefix) => `Swapped · tx ${txPrefix}…`,
    withdrawn: (txPrefix) => `Withdrawn · tx ${txPrefix}…`,
    errorTitle: "Something went wrong",
    errorRetry: "Try again",
    notFoundTitle: "Page not found",
    notFoundBody: "That route does not exist in the AMM demo.",
    notFoundHome: "Back home",
    help: {
      intro:
        "This client talks to the Anchor constant-product AMM (x · y = k) on the cluster configured by NEXT_PUBLIC_SOLANA_RPC_URL.",
      stepsTitle: "Suggested flow",
      steps: [
        "Connect Phantom (or another adapter wallet) to the same cluster as the app (localnet: http://127.0.0.1:8899).",
        "Create two SPL mints and fund your wallet (see scripts/fund-wallet.sh).",
        "Paste mint addresses under Active mints and Initialize.",
        "Initialize a pool with a unique seed and fee in basis points (e.g. 30 = 0.30%).",
        "Deposit liquidity (first deposit locks MINIMUM_LIQUIDITY LP tokens).",
        "Swap exact-in with a min-out slippage guard, or withdraw by burning LP.",
      ],
      fieldsTitle: "Key fields",
      fields: [
        "Seed — client seed used to derive the pool Config PDA. Same seed + program = same pool.",
        "Fee (bps) — swap fee on input; max 10_000 (100%).",
        "Min LP / Min out / Min X·Y — slippage protection; the tx reverts if the result is worse.",
        "Direction — X→Y spends mint X for Y; Y→X is the reverse.",
      ],
      phantomTitle: "Phantom on localnet",
      phantom: [
        "Enable Developer Mode in Phantom settings.",
        "Select Localhost or a custom RPC: http://127.0.0.1:8899.",
        "Copy your pubkey and run: ./scripts/fund-wallet.sh <PUBKEY>",
        "Then return here: Connect → Initialize → Deposit → Swap / Withdraw.",
      ],
      tipsTitle: "Tips",
      tips: [
        "Use the theme and language toggles in the header; preferences are saved in this browser.",
        "Amounts are raw token units (respect mint decimals, usually 6 in the demo).",
        "If a transaction fails, check that the wallet holds the required tokens and SOL for fees.",
      ],
    },
  },
  es: {
    brandCluster: (cluster) => `cluster ${cluster}`,
    themeToLight: "Cambiar a modo claro",
    themeToDark: "Cambiar a modo oscuro",
    themeLightLabel: "Claro",
    themeDarkLabel: "Oscuro",
    localeToEs: "Cambiar a español",
    localeToEn: "Switch to English",
    helpOpen: "Abrir ayuda",
    helpClose: "Cerrar ayuda",
    helpTitle: "Cómo usar esta demo del AMM",
    heroTitle: "Opera y aporta liquidez en tu pool local",
    heroBody:
      "Conecta una wallet, inicializa un pool de producto constante y luego deposita, intercambia o retira. Apunta NEXT_PUBLIC_SOLANA_RPC_URL a localnet o devnet.",
    mintsTitle: "Mints activos",
    mintsHint:
      "Pega las direcciones SPL de los mints de tu pool. Créalos con la CLI en localnet antes de depositar o intercambiar.",
    mintX: "Mint X",
    mintY: "Mint Y",
    initializeTitle: "Inicializar pool",
    depositTitle: "Depositar",
    swapTitle: "Intercambiar",
    withdrawTitle: "Retirar",
    seed: "Seed",
    feeBps: "Fee (bps)",
    amountX: "Cantidad X",
    amountY: "Cantidad Y",
    minLp: "LP mínimo",
    direction: "Dirección",
    swapXy: "X → Y",
    swapYx: "Y → X",
    amountIn: "Cantidad de entrada",
    minOut: "Mínimo de salida",
    lpAmount: "Cantidad LP",
    minX: "Mín. X",
    minY: "Mín. Y",
    actionInitialize: "Inicializar",
    actionDeposit: "Depositar",
    actionSwap: "Intercambiar",
    actionWithdraw: "Retirar",
    connectWalletFirst: "Conecta una wallet primero",
    invalidInput: "Entrada inválida",
    initializeFailed: "Falló la inicialización",
    depositFailed: "Falló el depósito",
    swapFailed: "Falló el intercambio",
    withdrawFailed: "Falló el retiro",
    initialized: (config, txPrefix) =>
      `Inicializado. Config ${config} · tx ${txPrefix}…`,
    deposited: (txPrefix) => `Depositado · tx ${txPrefix}…`,
    swapped: (txPrefix) => `Intercambiado · tx ${txPrefix}…`,
    withdrawn: (txPrefix) => `Retirado · tx ${txPrefix}…`,
    errorTitle: "Algo salió mal",
    errorRetry: "Reintentar",
    notFoundTitle: "Página no encontrada",
    notFoundBody: "Esa ruta no existe en la demo del AMM.",
    notFoundHome: "Volver al inicio",
    help: {
      intro:
        "Este cliente habla con el AMM Anchor de producto constante (x · y = k) en el cluster configurado por NEXT_PUBLIC_SOLANA_RPC_URL.",
      stepsTitle: "Flujo sugerido",
      steps: [
        "Conecta Phantom (u otra wallet) al mismo cluster que la app (localnet: http://127.0.0.1:8899).",
        "Crea dos mints SPL y fondea tu wallet (ver scripts/fund-wallet.sh).",
        "Pega las direcciones de mint en Mints activos e Inicializar.",
        "Inicializa un pool con un seed único y fee en basis points (p. ej. 30 = 0,30 %).",
        "Deposita liquidez (el primer depósito bloquea MINIMUM_LIQUIDITY tokens LP).",
        "Intercambia exact-in con protección min-out, o retira quemando LP.",
      ],
      fieldsTitle: "Campos clave",
      fields: [
        "Seed — semilla del cliente para derivar el PDA Config del pool. Mismo seed + programa = mismo pool.",
        "Fee (bps) — comisión del swap sobre la entrada; máximo 10_000 (100 %).",
        "LP mínimo / Mín. salida / Mín. X·Y — protección de slippage; la tx revierte si el resultado es peor.",
        "Dirección — X→Y gasta mint X por Y; Y→X es el inverso.",
      ],
      phantomTitle: "Phantom en localnet",
      phantom: [
        "Activa el modo desarrollador en ajustes de Phantom.",
        "Elige Localhost o un RPC personalizado: http://127.0.0.1:8899.",
        "Copia tu pubkey y ejecuta: ./scripts/fund-wallet.sh <PUBKEY>",
        "Vuelve aquí: Conectar → Inicializar → Depositar → Intercambiar / Retirar.",
      ],
      tipsTitle: "Consejos",
      tips: [
        "Usa los interruptores de tema e idioma en la cabecera; se guardan en este navegador.",
        "Las cantidades son unidades crudas del token (respeta los decimals del mint; suele ser 6 en la demo).",
        "Si una transacción falla, comprueba que la wallet tenga los tokens necesarios y SOL para fees.",
      ],
    },
  },
};

export const LOCALE_STORAGE_KEY = "amm-locale";
export const THEME_STORAGE_KEY = "amm-theme";

export function isLocale(value: string | null): value is Locale {
  return value === "en" || value === "es";
}
