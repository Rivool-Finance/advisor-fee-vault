export interface DuePosition { id: string; settledAt: bigint; interval: bigint; funded: boolean }
export interface KeeperBoundary {
  discover(): Promise<DuePosition[]>;
  now(): Promise<bigint>;
  read(id: string): Promise<DuePosition | null>;
  settle(id: string): Promise<string>;
}
export interface SettlementResult { position: string; status: "confirmed" | "reconciled" | "failed"; signature?: string; error?: string }

export async function runKeeper(rpc: KeeperBoundary): Promise<SettlementResult[]> {
  const now = await rpc.now();
  const results: SettlementResult[] = [];
  for (const initial of await rpc.discover()) {
    if (!initial.funded || now - initial.settledAt < initial.interval) continue;
    let signature: string | undefined;
    try {
      const current = await rpc.read(initial.id);
      if (!current || current.settledAt !== initial.settledAt || !current.funded) continue;
      signature = await rpc.settle(initial.id);
      results.push({ position: initial.id, status: "confirmed", signature });
    } catch (error) {
      if (error && typeof error === "object" && "signature" in error && typeof error.signature === "string") signature = error.signature;
      try {
        const current = await rpc.read(initial.id);
        if (!current || current.settledAt > initial.settledAt) {
          results.push({ position: initial.id, status: "reconciled", signature });
          continue;
        }
      } catch { /* Preserve the unresolved attempt when reconciliation also fails. */ }
      // Retry on the next scheduled run after fresh discovery, never blindly after a timeout.
      results.push({ position: initial.id, status: "failed", signature, error: error instanceof Error ? error.message : String(error) });
    }
  }
  return results;
}
