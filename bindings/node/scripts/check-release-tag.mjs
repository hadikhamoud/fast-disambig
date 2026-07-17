import { readFile } from 'node:fs/promises'

const tag = process.argv[2]
const cargo = await readFile(new URL('../../../Cargo.toml', import.meta.url), 'utf8')
const workspaceVersion = cargo.match(
  /\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1]

if (!workspaceVersion) {
  throw new Error('Could not read workspace version from Cargo.toml')
}
if (tag !== `v${workspaceVersion}`) {
  throw new Error(
    `Release tag ${tag} does not match Cargo version v${workspaceVersion}`,
  )
}

console.log(`Release tag matches Cargo version: ${tag}`)
