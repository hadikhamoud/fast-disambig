import { readFile, readdir, writeFile } from 'node:fs/promises'

const cargo = await readFile(new URL('../../../Cargo.toml', import.meta.url), 'utf8')
const workspaceVersion = cargo.match(
  /\[workspace\.package\][\s\S]*?^version\s*=\s*"([^"]+)"/m,
)?.[1]

if (!workspaceVersion) {
  throw new Error('Could not read workspace version from Cargo.toml')
}

async function updateJson(url, update) {
  const value = JSON.parse(await readFile(url, 'utf8'))
  update(value)
  await writeFile(url, `${JSON.stringify(value, null, 2)}\n`)
}

await updateJson(new URL('../package.json', import.meta.url), (manifest) => {
  manifest.version = workspaceVersion
})
await updateJson(new URL('../package-lock.json', import.meta.url), (lockfile) => {
  lockfile.version = workspaceVersion
  lockfile.packages[''].version = workspaceVersion
})

const npmRoot = new URL('../npm/', import.meta.url)
const npmDirectories = await readdir(npmRoot, { withFileTypes: true })
for (const directory of npmDirectories) {
  if (!directory.isDirectory()) continue
  await updateJson(
    new URL(`${directory.name}/package.json`, npmRoot),
    (manifest) => {
      manifest.version = workspaceVersion
    },
  )
}

console.log(`Synchronized npm packages to Cargo version ${workspaceVersion}`)
