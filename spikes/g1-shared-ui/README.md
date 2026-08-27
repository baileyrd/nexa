# G1 shared UI/loopback suitability spike

Disposable evidence code for Issue #116. It is not a production application or an architecture selection.

## Run

```bash
cd spikes/g1-shared-ui/frontend && npm ci && npm run check && npm test && npm run build
cd ../runtime && cargo test && cargo run -- --token g1-local-fixture
# browser: http://127.0.0.1:4173 (use `npm run preview`)
cd ../desktop && npm ci && npm run tauri build
```

The runtime listens on `127.0.0.1:43116` only. The deterministic development token is supplied explicitly to both clients. No real learner data or secrets are used. The desktop shell loads the exact frontend build and defines no Tauri commands or desktop business logic.
