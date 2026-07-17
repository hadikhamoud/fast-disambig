import { access } from 'node:fs/promises'

const targets = [
  ['darwin-arm64', 'fast-disambig.darwin-arm64.node'],
  ['darwin-x64', 'fast-disambig.darwin-x64.node'],
  ['linux-x64-gnu', 'fast-disambig.linux-x64-gnu.node'],
  ['linux-arm64-gnu', 'fast-disambig.linux-arm64-gnu.node'],
  ['linux-x64-musl', 'fast-disambig.linux-x64-musl.node'],
  ['win32-x64-msvc', 'fast-disambig.win32-x64-msvc.node'],
]

for (const [directory, binary] of targets) {
  await access(new URL(`../npm/${directory}/${binary}`, import.meta.url))
}

console.log(`Verified ${targets.length} native package artifacts`)
