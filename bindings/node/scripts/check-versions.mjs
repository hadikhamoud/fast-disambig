import { readFile } from 'node:fs/promises'

const cargo = await readFile(new URL('../../../Cargo.toml', import.meta.url), 'utf8')
const packageJson = JSON.parse(
  await readFile(new URL('../package.json', import.meta.url), 'utf8'),
)
const workspaceVersion = cargo.match(
  /\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1]

if (!workspaceVersion) {
  throw new Error('Could not read workspace version from Cargo.toml')
}
if (workspaceVersion !== packageJson.version) {
  throw new Error(
    `Version mismatch: Cargo ${workspaceVersion}, npm ${packageJson.version}`,
  )
}

console.log(`Cargo and npm versions match: ${workspaceVersion}`)
