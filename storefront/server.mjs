// Starts the SvelteKit server (adapter-node).
//
// Behind a reverse proxy on this host (PUBLISH_ADDR=127.0.0.1: the built-in Caddy or your own) the visitor's
// address is the entry the proxy appended to X-Forwarded-For. When the port is reachable directly, a visitor
// could forge that header, so the TCP connection's address is used instead.
// TRUST_PROXY=true|false overrides the automatic choice (auto = trust only with PUBLISH_ADDR=127.0.0.1).
const mode = (process.env.TRUST_PROXY || 'auto').trim().toLowerCase();
const behindProxy = mode === 'true' || (mode === 'auto' && (process.env.PUBLISH_ADDR || '').trim() === '127.0.0.1');

if (behindProxy) {
  process.env.ADDRESS_HEADER = 'X-Forwarded-For';
  process.env.XFF_DEPTH = '1';
} else {
  delete process.env.ADDRESS_HEADER;
  delete process.env.XFF_DEPTH;
}
console.log(`Client addresses: ${behindProxy ? 'from X-Forwarded-For (reverse proxy)' : 'from the TCP connection (no proxy trusted)'}`);

await import('./build/index.js');
