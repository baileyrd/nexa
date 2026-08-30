import { useEffect, useRef, useState } from 'react';

const BASE = 'http://127.0.0.1:43116/v1';
const TOKEN = 'g1-local-fixture';
type Phase = 'ready' | 'loading' | 'success' | 'cancelled' | 'disconnected' | 'error';

export function App() {
  const [phase, setPhase] = useState<Phase>('ready');
  const [message, setMessage] = useState('Ready.');
  const [hold, setHold] = useState(false);
  const abort = useRef<AbortController | null>(null);
  const socket = useRef<WebSocket | null>(null);

  const connect = () => {
    socket.current?.close();
    const ws = new WebSocket(`ws://127.0.0.1:43116/v1/events?token=${TOKEN}&version=1`);
    socket.current = ws;
    ws.onopen = () => {
      setPhase('ready');
      setMessage('Event connection ready.');
    };
    ws.onmessage = (event) => {
      const nextMessage = String(event.data);
      if (nextMessage === 'Cancellation acknowledged.') setPhase('cancelled');
      setMessage(nextMessage);
    };
    ws.onerror = () => {
      setPhase('error');
      setMessage('Event connection error. Use reconnect.');
    };
    ws.onclose = () => setPhase((current) => current === 'error' ? current : 'disconnected');
  };

  useEffect(() => {
    connect();
    return () => socket.current?.close();
  }, []);

  const run = async () => {
    abort.current = new AbortController();
    setPhase('loading');
    setMessage(hold ? 'Running held fixture. Press Cancel.' : 'Loading deterministic fixture…');
    try {
      const mode = hold ? '?mode=hold' : '';
      const response = await fetch(`${BASE}/fixture${mode}`, {
        headers: {
          Authorization: `Bearer ${TOKEN}`,
          'Nexa-Protocol-Version': '1',
          'Content-Type': 'application/json',
        },
        signal: abort.current.signal,
      });
      const body = await response.json() as {message?: string, error?: string};
      if (!response.ok) throw new Error(body.error ?? 'Request rejected');
      setPhase('success');
      setMessage(body.message ?? 'Fixture complete.');
    } catch (error) {
      if (error instanceof DOMException && error.name === 'AbortError') {
        if (socket.current?.readyState === WebSocket.OPEN) {
          setMessage('Cancellation requested; awaiting acknowledgement.');
        } else {
          setPhase('cancelled');
          setMessage('Request cancelled safely while events were disconnected.');
        }
      } else {
        setPhase('error');
        setMessage(`Recoverable error: ${error instanceof Error ? error.message : 'unknown'}`);
      }
    }
  };

  const cancel = () => {
    abort.current?.abort();
    if (socket.current?.readyState === WebSocket.OPEN) {
      socket.current.send(JSON.stringify({type: 'cancel', request_id: 'fixture'}));
    }
  };

  return <main>
    <a className="skip" href="#controls">Skip to controls</a>
    <header><p className="eyebrow">Disposable evidence · Issue #120</p><h1>Nexa shared client fixture</h1><p>One static text path is used in browser and desktop. Animation is never required.</p></header>
    <section id="controls" aria-labelledby="fixture-title">
      <h2 id="fixture-title">Lifecycle fixture</h2>
      <label><input type="checkbox" checked={hold} onChange={(event) => setHold(event.target.checked)} /> Hold for interactive cancellation</label>
      <div className="controls"><button onClick={run}>Run fixture</button><button onClick={cancel} disabled={phase !== 'loading'}>Cancel</button><button onClick={connect}>Reconnect</button></div>
      <div className={`status ${phase}`} role={phase === 'error' ? 'alert' : 'status'} aria-live="polite"><strong>{phase}</strong><span>{message}</span></div>
    </section>
    <details><summary>Accessible static details</summary><p>The fixture reports loading, success, cancellation, disconnect, and recoverable error states as text.</p></details>
  </main>;
}
