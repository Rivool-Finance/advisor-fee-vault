import { copyFileSync, mkdirSync } from "node:fs";
mkdirSync("idl", { recursive: true });
for (const name of ["advisor_fee_vault", "stand_in_yield"]) {
  copyFileSync(`target/idl/${name}.json`, `idl/${name}.json`);
  copyFileSync(`target/types/${name}.ts`, `idl/${name}.ts`);
}
