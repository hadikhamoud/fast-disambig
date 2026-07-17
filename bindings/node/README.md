# fast-disambig for Node.js

Native Node.js bindings for fast Arabic morphological disambiguation and stemming.

```ts
import { camel } from 'fast-disambig'

const stemmer = await camel.Stemmer.create()
const result = await stemmer.stem('والكتاب الجميل')
```

Heavy operations and model loading have asynchronous APIs. Sync variants are available for scripts. See the [project README](https://github.com/hadikhamoud/fast-disambig#nodejs) for the full API and Next.js setup.
