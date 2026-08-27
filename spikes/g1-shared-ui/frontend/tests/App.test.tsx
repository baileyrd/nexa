import '@testing-library/jest-dom/vitest';
import {act, cleanup, fireEvent, render, screen, waitFor} from '@testing-library/react';
import {afterEach, beforeEach, expect, test, vi} from 'vitest';
import {App} from '../src/App';

class MockWebSocket {
  static OPEN = 1;
  static instances: MockWebSocket[] = [];
  readyState = 0;
  onopen = () => {};
  onmessage = () => {};
  onerror = () => {};
  onclose = () => {};
  send = vi.fn();
  close = vi.fn();
  constructor() { MockWebSocket.instances.push(this); }
}

beforeEach(() => {
  MockWebSocket.instances = [];
  vi.stubGlobal('WebSocket', MockWebSocket);
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

function socket() { return MockWebSocket.instances.at(-1)!; }

test('exposes keyboard controls and static path', () => {
  render(<App />);
  expect(screen.getByRole('button', {name: 'Run fixture'})).toBeEnabled();
  expect(screen.getByText('Accessible static details')).toBeVisible();
  expect(screen.getByRole('status')).toHaveTextContent('Ready.');
});

test('reports loading and success', async () => {
  let resolveFetch!: (value: Response) => void;
  vi.stubGlobal('fetch', vi.fn(() => new Promise(resolve => { resolveFetch = resolve; })));
  render(<App />);
  fireEvent.click(screen.getByRole('button', {name: 'Run fixture'}));
  expect(screen.getByRole('status')).toHaveTextContent('loading');
  resolveFetch(new Response(JSON.stringify({message: 'Deterministic fixture complete.'}), {status: 200}));
  await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('success'));
});

test('aborts safely without sending on a connecting socket', async () => {
  vi.stubGlobal('fetch', vi.fn((_url, init) => new Promise((_resolve, reject) => {
    init.signal.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError')));
  })));
  render(<App />);
  fireEvent.click(screen.getByRole('button', {name: 'Run fixture'}));
  fireEvent.click(screen.getByRole('button', {name: 'Cancel'}));
  await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('cancelled'));
  expect(socket().send).not.toHaveBeenCalled();
});

test('sends cancellation acknowledgement request on an open socket', () => {
  vi.stubGlobal('fetch', vi.fn(() => new Promise(() => {})));
  render(<App />);
  socket().readyState = MockWebSocket.OPEN;
  fireEvent.click(screen.getByRole('button', {name: 'Run fixture'}));
  fireEvent.click(screen.getByRole('button', {name: 'Cancel'}));
  expect(socket().send).toHaveBeenCalledWith('{"type":"cancel","request_id":"fixture"}');
});

test('reports disconnect and reconnect', () => {
  render(<App />);
  act(() => socket().onclose());
  expect(screen.getByRole('status')).toHaveTextContent('disconnected');
  fireEvent.click(screen.getByRole('button', {name: 'Reconnect'}));
  act(() => socket().onopen());
  expect(screen.getByRole('status')).toHaveTextContent('Event connection ready.');
});

test('reports recoverable request and socket errors', async () => {
  vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({error: 'rejected'}), {status: 401})));
  render(<App />);
  fireEvent.click(screen.getByRole('button', {name: 'Run fixture'}));
  await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('Recoverable error: rejected'));
  act(() => socket().onerror());
  expect(screen.getByRole('alert')).toHaveTextContent('Event connection error. Use reconnect.');
});
