# BlackLab Query Interface

A modern React UI for querying BlackLab corpus search engine instances.

## Features

- **Full BCQL Support**: Query using BlackLab Corpus Query Language (CorpusQL)
- **Advanced Search Options**: Filter, sample, sort, and group results
- **Pagination**: Navigate through large result sets
- **Export Functionality**: Download results as JSON, CSV, or TXT
- **Query History**: Keep track of recent searches
- **Real-time Stats**: View hit counts, search time, and document statistics
- **Responsive Design**: Works on desktop and mobile devices

## Getting Started

### Prerequisites

- Node.js (v14 or higher)
- npm or yarn

### Installation

```bash
cd blacklab-query-ui
npm install
```

### Development

```bash
npm run dev
```

This will start the development server at `http://localhost:5173`

### Build for Production

```bash
npm run build
```

The built files will be in the `dist` directory.

### Preview Production Build

```bash
npm run preview
```

## Configuration

Update the default server URL and corpus name in `src/App.jsx`:

```javascript
const [serverUrl, setServerUrl] = useState('http://10.250.0.37:8080/blacklab-server')
const [corpus, setCorpus] = useState('dhd-index')
```

You can also change these values directly in the UI.

## Query Examples

### Simple Searches
- `"word"` - Search for a specific word
- `"(wo)?man"` - Use regex patterns (man or woman)

### Annotations
- `[lemma="search" & pos="noun"]` - Search by lemma and part-of-speech
- `[pos="ADJ"]+ "man"` - One or more adjectives before "man"

### Spans
- `<s/>` - Find all sentences
- `<s> containing "cat"` - Sentences containing "cat"
- `"baker" within <person/>` - Word within a named entity

### Capture Constraints
- `A:[] "by" B:[] :: A.word = B.word` - Repeated words (e.g., "day by day")

## API Parameters

### Basic Parameters
- **Pattern**: The query pattern in BCQL syntax
- **Pattern Language**: corpusql (default) or contextql
- **Number**: Results per page
- **Words Around Hit**: Context words to show
- **First**: Starting position for pagination

### Advanced Options
- **Filter**: Filter by document metadata
- **Sample**: Random sample size (number or percentage)
- **Sort By**: Sort results by annotation (e.g., `hit:word`)
- **Group By**: Group results by annotation
- **With Spans**: Include span information in results

## CORS Issues

If you encounter CORS errors when querying your BlackLab server, you have two options:

### Option 1: Configure BlackLab Server
Add CORS headers to your BlackLab server configuration.

### Option 2: Use a Proxy
Create a `vite.config.js` file:

```javascript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/blacklab-server': {
        target: 'http://10.250.0.37:8080',
        changeOrigin: true,
      }
    }
  }
})
```

Then update the server URL in the app to use the relative path:
```javascript
const [serverUrl, setServerUrl] = useState('/blacklab-server')
```

## Learn More

- [BlackLab Documentation](https://inl.github.io/BlackLab/)
- [BCQL Query Language Guide](https://inl.github.io/BlackLab/guide/query-language.html)
- [React Documentation](https://react.dev)
- [Vite Documentation](https://vitejs.dev)

## License

MIT
# fast-disambig
