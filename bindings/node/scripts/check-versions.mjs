import { readFile, readdir } from 'node:fs/promises'

const cargo = await readFile(new URL('../../../Cargo.toml', import.meta.url), 'utf8')
const packageJson = JSON.parse(
  await readFile(new URL('../package.json', import.meta.url), 'utf8'),
)
const packageLock = JSON.parse(
  await readFile(new URL('../package-lock.json', import.meta.url), 'utf8'),
)
const workspaceVersion = cargo.match(
  /\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1]

if (!workspaceVersion) {
  throw new Error('Could not read workspace version from Cargo.toml')
}
const versions = [
  ['package.json', packageJson.version],
  ['package-lock.json', packageLock.version],
  ['package-lock.json root package', packageLock.packages?.['']?.version],
]
const npmDirectories = await readdir(new URL('../npm', import.meta.url), {
  withFileTypes: true,
})
for (const directory of npmDirectories) {
  if (!directory.isDirectory()) continue
  const manifest = JSON.parse(
    await readFile(
      new URL(`../npm/${directory.name}/package.json`, import.meta.url),
      'utf8',
    ),
  )
  versions.push([`npm/${directory.name}/package.json`, manifest.version])
}

for (const [source, version] of versions) {
  if (version !== workspaceVersion) {
    throw new Error(
      `Version mismatch in ${source}: Cargo ${workspaceVersion}, npm ${version}`,
    )
  }
}

console.log(`Cargo and npm versions match: ${workspaceVersion}`)
