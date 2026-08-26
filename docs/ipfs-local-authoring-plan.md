# Local Authoring + P2P Exchange Plan

Users author in the browser (**IndexedDB**). They **exchange collections peer-to-peer** and **keep replicas in sync with each other** without Lensisku in the path. Lensisku PostgreSQL remains optional (canonical dictionary + discovery). Users may also publish a copy to **ATProto** (federation) if they want a PDS-backed `at://` identity.

**P2P is the default path.** Federation and central pin are backups, not the way two people share a collection when both are online.

| Backend | Kind | Share id | Role in this product |
|---|---|---|---|
| **IPFS + OrbitDB (Helia)** | P2P log + content-addressed blocks | `/orbitdb/zdpu…` | **Maximum browser P2P:** live replica sync (WebRTC / circuit-relay) |
| **iroh** | P2P blobs (BLAKE3) | blob ticket / `/p2p/…` | Overlay + Lensisku pin; **browsers are relay-only** (WASM) |
| **AT Protocol** | Federation (HTTPS to a PDS) | `at://did/…/rkey` | Optional identity/backup; **not** peer-to-peer |
| **Yggdrasil** | Encrypted IPv6 **mesh underlay** | `200::/7` node address (from pubkey) | **Not a collection store.** Optional native-path transport (NAT traversal as “plain IPv6”). Browsers cannot join the public mesh without a daemon or experimental WASM + **WSS peers**. |

Backends are not interchangeable on the wire. Import copies into IndexedDB; the user can republish to another backend. One **owner** writes; everyone else **replicates and stays synced** (follow), or **forks** into their own collection. No merge of two writers.

The Rust process: Helia-facing pin if we run rust-ipfs/kubo **or** we treat Helia as browser-only and pin via a small libp2p peer — prefer **in-process iroh relay + iroh pin** for iroh, and for OrbitDB **either** a Rust libp2p/Helia-compatible pin node **or** “enough online peers + optional Kubo”. Be honest in ops: OrbitDB max-P2P in browsers still needs **circuit-relay / WebRTC signaling**; we should run that **on the same Lensisku origin** (same process if a Rust circuit-relay exists, else same host path). ATProto AppView is optional indexer, not a PDS.

---

## Goal

1. **Local-first** IndexedDB; works with no network
2. **P2P first:** two (or more) users exchange a collection and **sync replicas** while they are online, **without** Lensisku, Bluesky, or n0 if a direct/WebRTC path exists
3. **Single writer** per collection; replicas follow the owner’s log (not a multi-writer merge)
4. **Survive our server down:** already-shared collections keep working between peers who have the address and at least one replica online (or a public IPFS/iroh peer)
5. Optional **ATProto** copy for stable `at://` and PDS hosting
6. Optional **sync into PostgreSQL** with a diff
7. Discovery catalog is a convenience, **not a gatekeeper**
8. Rust API: iroh node + **in-process iroh-relay** on the existing origin; OrbitDB path: browser Helia + our circuit-relay/WebRTC helper on that origin; ATProto: AppView only

---

## Federation, distributed storage, and P2P (what we actually want)

Three *storage* ideas get sold as “decentralized.” Only **OrbitDB/Helia** lets two Lensisku **browser** users **swap a collection and keep it in sync** while our servers are off. **Yggdrasil** is a different layer (IPv6 reachability), not a store.

### Federation (ATProto, ActivityPub, email)

Many **servers** speak one protocol. Your data lives on **your home server** (PDS). Other servers crawl or fetch it. Clients are ordinary HTTPS apps.

- **Up when:** your PDS is up (and maybe a relay/AppView for discovery).
- **Down when:** your PDS is down — **peers cannot fetch from you**, even if both browsers are online.
- **Not P2P:** Alice’s tab never talks to Bob’s tab. Both talk to servers.
- **Resists Lensisku down:** yes for **reads of already-crawled** data if we cached it, and yes for ATProto if Bob fetches Alice’s PDS directly. **No** for “two browsers sync with each other.”
- **Use for:** stable `at://` identity, backup, people who already have Bluesky; **not** as the P2P exchange path.

### Overlay IPv6 mesh (Yggdrasil)

Every node is a **router**. You get a cryptographic IPv6 address and encrypted multi-hop paths. There is **no collection object**, no Bitswap, no `at://`. It answers “can this host reach that host despite NAT / no public IPv6?” not “follow this dictionary.”

- **Up when:** both ends run a Ygg **daemon** (or yggstack) and share a connected graph (LAN multicast, static peers, or the **public testnet** via volunteer peers).
- **Down when:** no peering (isolated node), or browser-only with no WASM/WSS path.
- **Not a product backend:** Alice’s tab still does not speak Ygg. Native Alice↔Bob IPv6 is P2P at L3; the SPA is unchanged.
- **Resists Lensisku down:** **yes** for two daemons on the public mesh. **No** extra help for two tabs.
- **Use for:** optional native underlay, optional `200::` bind of our API; **not** as the share id or live-sync fabric. Details: **Yggdrasil** section below.

### Distributed file system / content-addressed store (IPFS, iroh-blobs)

Files (or blocks) named by **hash**. Any node that has the bytes can serve them. There is no single file server, but there is still **discovery** (DHT, tickets, providers) and often **relays** for NAT.

- **IPFS (Helia / Kubo):** SHA-256 CIDs, Bitswap, public DHT. Classic “put this on the swarm.” Browsers cannot speak UDP DHT well; they use **WebRTC, WebSockets, and circuit-relay**.
- **iroh:** same *idea* (content-addressed blobs), different hash (BLAKE3) and overlay. **Not** the public IPFS network. Native nodes hole-punch; **browser WASM is always relayed**.
- **Up when:** at least one provider of that hash is reachable (peer, pin node, or gateway).
- **Resists Lensisku down:** **yes** if another peer still has the blocks (or public IPFS / n0+iroh). **No** if the only copy was our pin and every peer closed the tab.
- **Sync:** a **new hash** is a new object unless you add a mutable pointer (IPNS, OrbitDB address, iroh-docs, our `collection_key`). A one-shot blob is **exchange**, not **live sync**.

### P2P database / append-only log (OrbitDB on IPFS)

OrbitDB is a **single-writer log** (CRDT Merkle-CRDT) stored as IPFS blocks, gossiped over **libp2p pubsub**. The **manifest address does not change** when the owner appends. Subscribers **replicate and stay updated** as long as they can find the owner or another replica.

That is the product behavior we want for “users exchange collections and sync them between themselves”:

1. Owner publishes → share `/orbitdb/…` (or a Lensisku URL that wraps it).
2. Peer opens the address → Helia finds the owner (WebRTC) or a replica (Bitswap) → **live follow**.
3. Owner edits → peers’ replica DBs update **without** Lensisku or a PDS.
4. Peer may **fork** (copy into their IndexedDB and publish a new DB they own).

**Resists Lensisku down:** **yes** for live sync if peers can still connect (WebRTC or any circuit-relay still up — ideally **ours**, but any reachable relay, or a third replica). If **all** relays and **all** replicas are gone, sync stops until someone with the blocks comes back.

### What “maximum P2P” means here

| Situation | Max-P2P (OrbitDB/Helia) | iroh in the browser | ATProto | Yggdrasil |
|---|---|---|---|---|
| Alice and Bob both online, Lensisku **down** | Sync if WebRTC works or **some** circuit-relay exists | Sync only if **some iroh relay** exists (ours is down → n0) | Sync via Alice’s **PDS**, not via Bob’s tab | Native daemons: yes if both still peered (public peers ≠ us). Tabs: no |
| Alice and Bob both online, **all** our servers down, n0 down, no PDS | WebRTC **direct** still works | **Fails** (WASM cannot hole-punch) | **Fails** | Native: yes via volunteer/public peers or LAN. Tabs: no |
| Alice offline, Bob online, our pin up | Bob reads from pin / IPFS | Bob reads from our blob pin | Bob reads PDS / our AppView cache | Irrelevant unless the pin process is also a Ygg node Bob can route to |
| Nobody online, only hashes on a DHT | Possible if Kubo/public IPFS still has blocks | Possible if some iroh provider remains | PDS disk still has records | Ygg has **no content DHT**; addresses are nodes, not files |

**Implication:** to maximize P2P exchange + ongoing replica sync in **this** (browser) app, **IPFS + OrbitDB via Helia is the primary overlay**. iroh is a strong second overlay (especially native/server pin, tickets) but **cannot** match Helia WebRTC when every relay is dead. ATProto is **federation**, kept as an optional store, not as the peer-sync fabric. **Yggdrasil is an underlay**, not a fourth publish backend: it does not hash collections, keep a live log, or give two *tabs* a share id. Native Helia/iroh/HTTP could *ride* Ygg IPv6 if both ends run a daemon; that is a power-user path, not the SPA default.

### Pros and cons

**IPFS + OrbitDB (Helia in the browser)**

- Pros: stable `/orbitdb/` address while the owner appends; **live replica sync**; Bitswap from any peer who has blocks; WebRTC can be **true browser-to-browser** with no Lensisku; public IPFS can keep blocks if anyone pins; mature JS stack for this exact “follow a DB” UX.
- Cons: **not** iroh (separate overlay); browsers still often need **circuit-relay + signaling**; DHT is weak in-browser; Kubo/Helia version and gossipsub config are fiddly; identity key in IndexedDB; “IPFS is forever” is false without a pin; rust-only pin means rust-ipfs or Kubo sidecar (Kubo is Go — conflict with “no Node” is OK, Go daemon is still extra); libp2p ≠ iroh-relay.

**iroh (`iroh-blobs` + in-process relay)**

- Pros: one Rust process; good NAT for **native** nodes; tickets; we control a relay on the same origin; pin in-process; n0 as extra relays.
- Cons: **WASM = 100% relayed** → if Lensisku **and** n0 are down, **two browsers cannot sync**; snapshot hash changes unless we add a pointer; not IPFS-compatible; wasm-pack cost.

**ATProto**

- Pros: stable `at://`; account portability; PDS holds data while the author is offline (better than “tab closed, blob gone”); `did` identity; optional firehose discovery.
- Cons: **not P2P**; PDS outage blocks publish and often fetch; firehose is public; ~100 KB records; OAuth; PLC/`bsky.social` coupling; **does not** satisfy “sync collections between themselves” except as “both fetch the same PDS.”

**Yggdrasil (IPv6 overlay mesh)**

- Pros: true end-to-end encrypted IPv6 between daemons; **stable address** derived from the node key (roams with the node); NATed machines become *reachable* once they have **any** peering (outbound to a public peer is enough to join — no inbound port-forward required for *using* the network); LAN multicast auto-peering; public testnet exists so **users do not need us** to join; any IPv6 app can run on top (HTTP, SSH, even Kubo/Helia on native); self-heals when a link dies; no central operator of the routing plane.
- Cons: **not** IPFS/OrbitDB/iroh — no CIDs, no replica log, no “follow this collection”; **alpha** research routing (wire format can still change); **not anonymous** (direct Internet peers see your IP); joining the global net **bridges** your node to everyone (do not treat it as a private VPN); home nodes with many distant peers can **carry other people’s transit**; needs a **TUN** (or yggstack SOCKS) — privilege/`CAP_NET_ADMIN` for the official daemon; **no native browser stack** in yggdrasil-go (experimental WASM/VTun exists; most public `wss://` peers **reject CORS** as of 2026); still need an application protocol on top; firewall required because you are globally reachable on `200::/7`.

**Lensisku PostgreSQL (classic server)**

- Pros: search, accounts, quotas, always-on if we are up.
- Cons: **zero P2P**; if we are down, server collections are down; does not replace overlay sync.

### How peer sync works (product)

```
Owner (IndexedDB) --publish--> OrbitDB log (and/or iroh blob)
Peer opens address --> replicate --> IndexedDB mirror (read-only follow)
Owner edits --> pubsub/bitswap --> peer replica updates
Peer clicks Fork --> new local collection they own --> they publish their own address
```

“Sync them between themselves” = **follow the owner’s log on the overlay**, not CRDT-merge two owners. Devices of the **same** owner: same OrbitDB identity (exported key) or re-publish from IndexedDB.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Lensisku (optional for P2P exchange; same Actix origin)         │
│   Catalog, PostgreSQL sync, iroh pin, iroh-relay /relay         │
│   libp2p circuit-relay + WebRTC signaling for Helia (same host) │
│   ATProto AppView (optional backup index)                       │
│   Optional: Ygg daemon (200:: API) or WSS peer — not required   │
└──────┬────────────────────┬─────────────────────┬───────────────┘
       │ pin / relay        │ signaling / relay     │ index at://
┌──────┴────────┐  ┌────────┴──────────┐  ┌────────┴─────────────┐
│ OrbitDB/Helia │  │ iroh overlay      │  │ User PDS (ATProto)   │
│ live replica  │  │ blobs + tickets   │  │ federated store      │
│ WebRTC/P2P    │  │ browser→/relay    │  │ not peer tabs        │
└──────┬────────┘  └────────┬──────────┘  └────────┬─────────────┘
       └────────────────────┴──────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────────┐
│ IndexedDB — primary; follow/fork from whichever overlay         │
└─────────────────────────────────────────────────────────────────┘
```

**Publish picker:** Local · **OrbitDB (P2P sync)** · iroh · ATProto · any combination. Default for “share with a friend now” = **OrbitDB**. iroh and ATProto are extra copies.

### Publish model — OrbitDB / Helia (primary P2P)

1. Helia + OrbitDB **Documents** (or KeyValue), **IPFSAccessController**, owner-only writes.
2. Address `/orbitdb/zdpu…` is **stable** as the log grows. Peers `orbitdb.open(address)` and **subscribe**; items appear as the owner appends.
3. Same JSON fields as the snapshot `items[]` / meta `__meta__` doc.
4. Transports: WebRTC for browser-to-browser; WebSockets to **our** circuit-relay on the Lensisku origin (same host as `/api`). When Lensisku is down, WebRTC still works if public STUN/ICE succeeds.
5. Optional: pin the root CID on a long-lived node so the collection survives “everyone closed the tab.”

### Publish model — iroh

Single-owner, read-only for others — no CRDT merge.

1. **Snapshot blob:** JSON `{ meta, items }` hashed with BLAKE3. Share a **blob ticket**. Editing produces a **new** hash.
2. **Stable pointer:** discovery row `(username, collection_key, current_ticket)` so `https://<origin>/p2p/<collection-key>` tracks the latest ticket.

### Publish model — ATProto

1. User **signs in with ATProto** (OAuth + DPoP) to their PDS (`bsky.social` or self-hosted).
2. Lensisku lexicon under a domain we control, e.g. `org.lojban.lensisku.collection` (meta, rkey = `collection_key`) and `org.lojban.lensisku.item` (**one record per definition**, rkey = item `id`). Bluesky-hosted PDSes cap a record at ~**100 KB** — do not put the whole collection in one record.
3. Share `at://<did>/org.lojban.lensisku.collection/<collection_key>` or `https://<origin>/at/<handle-or-did>/<collection_key>`. **Edits keep the same URI** (`putRecord`).
4. Records on a PDS repo are **public** on the firehose. “Private” in the UI means “not listed in Lensisku discovery,” not encrypted.
5. Publish requires a **reachable PDS**. Drafts stay in IndexedDB offline; the UI queues or blocks publish until online.

Fork: import to IndexedDB, then publish with **your** OrbitDB identity, iroh key, and/or DID.

---

## iroh relays

An iroh relay is a **stateless** HTTPS/WebSocket (optional QUIC) service that:

1. Helps native endpoints hole-punch
2. Forwards encrypted packets if a direct path never comes up
3. Carries **all** browser traffic (WASM has no UDP)

Payload is E2E encrypted. Relays see IPs, timing, bytes, endpoint IDs. Native networks often go direct later; **browsers never do**.

### n0 public relays (fallback only)

From iroh `defaults::prod` (confirm at crate upgrade):


| Region       | Hostname                    |
| ------------ | --------------------------- |
| NA East      | `use1-1.relay.n0.iroh.link` |
| NA West      | `usw1-1.relay.n0.iroh.link` |
| EU Central   | `euc1-1.relay.n0.iroh.link` |
| Asia-Pacific | `aps1-1.relay.n0.iroh.link` |


Policy: hobby/dev, **no SLA**, shared, rate-limited, latest stable iroh only. Metadata visible to n0. Use as backup when our process is down or saturated — not as the availability story.

### Lensisku relay (required, in-process)

**Same Rust process as Actix. Same public origin as the API already uses. No `relay.` subdomain. No second systemd unit.**

iroh clients use a **RelayUrl** that is the HTTP origin; they connect to `/relay` (legacy `/derp`). If Actix is already reached as `https://lensisku.lojban.org` (or whatever production already is), the relay URL is that origin. Nginx/Caddy in front of `:8080` must **proxy WebSocket** for `/relay` and `/derp` to the same upstream as `/api`.

**How to embed**

1. Enable `iroh-relay` with the `server` feature. Use the public `RelayService` / HTTP pieces (see current `iroh-relay` `http_server`: extra handlers on the same listener, default `/relay` + `/derp`).
2. Mount the relay WebSocket on the **existing** `HttpServer` (or drive Actix routes from the relay server’s `request_handler` for `/api` and the rest). One TCP bind, one process.
3. **Shared relay state across Actix workers.** `HttpServer` runs many workers; the client registry must be a single `Arc` (`web::Data`), not a map per worker, or peers never see each other.
4. Optional: same process binds **UDP QUIC** on the host (default ~7824) for native hole-punch helpers. Browsers do not use it. Not a new hostname.
5. Access policy: start open enough that anonymous **readers** can connect, or accept that readers use `GET /api/p2p/blob/:ticket` (HTTPS fallback) and only publishers use the relay. Connection caps and metrics on the shared process.
6. Client relay map: `[<this origin>, …n0 prod]`. WASM and the server endpoint use the same map. `GET /api/p2p/relay-map` can serve it.

**TLS:** production stays as today (reverse proxy on 443 → Actix). Relay WebSockets must survive that proxy (Upgrade headers, long-lived connections, no body buffering).

The relay does **not** store collections. Pinning is `iroh-blobs` in the same process. If the process dies, both API and relay die; n0 is the fallback for overlay, and pinned blobs on disk come back when the process restarts.

**Discovery vs relay:** n0 DNS/PKARR (`dns.iroh.link`) is separate. MVP tickets should embed provider addressing so we do not depend on n0 DNS.

---

## Helia / OrbitDB (maximum P2P)

Browser stack: `helia`, `@orbitdb/core`, `@libp2p/webrtc`, `@libp2p/websockets`, `@libp2p/circuit-relay-v2`, gossipsub. This overlay **does not** use `/relay` (iroh). It needs:

- **WebRTC** for tab-to-tab (STUN; optionally TURN on the same Lensisku host if we add it — still not iroh).
- **Circuit-relay v2** on the **same public origin** as the API (path such as `/p2p-ws`), so NATed browsers can meet when WebRTC fails. Same “no extra subdomain” rule as iroh-relay. Prefer a Rust libp2p relay in-process; if that is not mature enough, a small daemon on the **same machine and path** via the reverse proxy — last resort.

Pinning OrbitDB for “everyone closed the tab”: long-lived Helia or Kubo that `orbitdb.open` the address, **or** pin CIDs. rust-ipfs is the Rust-shaped option; Kubo is operationally heavier. Until that exists, **at least one follower online** (or iroh/ATProto copy) is the durability story.

Helia and iroh **cannot** replicate each other’s blocks. Dual publish means two copies.

Full browser architecture, composables, routes, and tests: **Frontend — OrbitDB / Helia** below.

---

## ATProto (user PDS)

AT Protocol stores public JSON records in a per-account **repository** on a **PDS**. Clients write over HTTPS XRPC. **Relays** crawl PDSes into a firehose; **Jetstream** is a JSON subset. An **AppView** indexes a lexicon slice. This is federation, not iroh-style packet relay.

**Lensisku role:** AppView only. Filter Jetstream (or `subscribeRepos`) for `org.lojban.lensisku.*`, store in PostgreSQL, serve discovery and HTTPS read fallback. Writes: **browser OAuth → user’s PDS**. We do not run `@atproto/pds` (Node) inside Actix.

**Identity:** DID (`did:plc` or `did:web`) + handle. “Sign in with ATProto” is separate from the Lensisku password account; link them when registering for discovery.

**Infra stability (no SLA):**

| Piece | Typical host | If down |
|---|---|---|
| User PDS | `*.bsky.social` or self-host | Cannot **publish**; IndexedDB drafts survive. Self-host writes can stay up on a VPS. |
| Relay / Jetstream | `bsky.network`, `jetstream.us-east.bsky.network`, others | Our indexer misses **new** records until catch-up. Self-hosted PDSes sometimes **desync** (seq/cursor); data is on the PDS but AppViews never see it until `requestCrawl`. |
| `api.bsky.app` | Bluesky AppView | Irrelevant to our lexicons. Do not use it for dictionary reads. |
| `plc.directory` | `did:plc` registry | Handle/DID resolution fails unless we run a replica. |
| Our AppView + DB | this process | We still serve **already indexed** copies. |

Bluesky the product has had multi-hour outages and DDoS (e.g. April 2026). Independent AppViews can stay up. n0 iroh relays are hobby-grade; Bluesky PDS/AppView is production-ish **most days**, still without an SLA.

**Resilience:** cache indexed records (same idea as iroh pin); optional PLC replica; HTTPS `GET` of records we already ingested; document that new ATProto publishes need the author’s PDS.

Lexicon authority: DNS `_lexicon` TXT on a domain we control; publish `com.atproto.lexicon.schema` records.

---

## Yggdrasil (optional underlay — not a publish backend)

[Yggdrasil](https://yggdrasilnetwork.org/) is a **userspace IPv6 router**: spanning-tree + greedy routing, traffic **E2E encrypted**, each node a `200::/7` address from its public key. Peerings are TCP / TLS / QUIC / **WebSocket** (`ws://`, `wss://` since 0.5.7) over the ordinary Internet or a LAN. It is closer to **cjdns / a mesh VPN** than to IPFS. Overlay-by-convenience: the *product* we want (collection follow/fork) still lives in OrbitDB, iroh, or ATProto.

### Do we have to run our own servers?

| Goal | Run Lensisku-owned Ygg nodes? |
|---|---|
| **Users join the public mesh** (native app, desktop, Android) | **No.** Install [yggdrasil-go](https://github.com/yggdrasil-network/yggdrasil-go), add **2–3 nearby** [public peers](https://publicpeers.neilalexander.dev/) (`Peers:` in `yggdrasil.conf`). Volunteer operators already listen. Default config does **not** accept inbound peerings; you are a leaf unless you set `Listen`. |
| **Private club / never touch the public testnet** | **Yes, our (or the users’) nodes only.** Static `Peers` + `AllowedPublicKeys`. **Do not** also peer with a public peer — that **bridges** the private mesh onto the global net. |
| **Always-on Lensisku over Ygg IPv6** (catalog, pin, HTTP API at a `200::` address) | **Optional one node** on the existing VPS: yggdrasil-go (or yggstack) beside Actix, firewall the TUN, publish the Ygg AAAA / `.pk.ygg` for people already on the mesh. This is **federation over a mesh**, not tab-to-tab sync. We still need the Internet-facing origin for everyone else. |
| **Browser tabs join Ygg without a local daemon** | **Yes, if we want this at all:** a **WSS listener we control** (CORS + path on the Lensisku origin, e.g. `/ygg-ws`), because experimental in-page WASM ([asciimoth/ygg](https://github.com/asciimoth/ygg) demo) cannot use most public WebSocket peers. That is another long-lived WebSocket on the API process, same class of ops as iroh `/relay` and libp2p `/p2p-ws`. **Not recommended as a product path** — Helia WebRTC already covers “two browsers, no extra OS install.” |
| **SOCKS into Ygg from the SPA** | Users run **[yggstack](https://github.com/yggdrasil-network/yggstack)** locally (no TUN). The website cannot open a system SOCKS proxy. Useless as a default. |

**Bottom line:** for the **IndexedDB + Helia** plan we do **not** need Yggdrasil servers. Public peers are enough for *native* users who opt in. We would only run our own Ygg process if we want (a) a stable `200::` API/pin for mesh users, (b) a private mesh, or (c) a CORS-friendly WSS peer for experimental WASM — (c) duplicates work we already plan for circuit-relay.

### How it sits next to Helia / iroh

```
Yggdrasil          = IPv6 reachability (who can talk to whom)
libp2p / iroh      = overlay sessions + (for IPFS) blocks
OrbitDB            = mutable single-writer log
ATProto / Postgres = servers
```

Two people with Ygg daemons can TCP to each other’s node addresses **without STUN**, which is the NAT story Ygg is good at. Two **Lensisku tabs** still cannot, unless WASM+WSS or a local helper. Helia-in-browser remains the max-P2P path when Lensisku is down.

| Situation | Yggdrasil (native daemons) | OrbitDB/Helia in the tab |
|---|---|---|
| Alice & Bob online, Lensisku down | Works if both are peered into the **same** mesh (public or private) | WebRTC / any circuit-relay |
| Only browsers, no daemon | **Fails** (unless experimental WASM + WSS peer) | Designed for this |
| Share id for a collection | Ygg address is a **machine**, not a DB | `/orbitdb/zdpu…` |
| Alice offline | Her node is gone unless something else hosts the app/data | Bitswap from pin / other replica |

### If we ever ship a native helper

1. Document: install Ygg, 2–3 close public peers, firewall TUN (`ip6tables`/`ufw` examples from the [FAQ](https://yggdrasil-network.github.io/faq.html)).
2. Optional: Lensisku `Listen` + `AllowedPublicKeys` **off** unless we intend to be a public peer (transit + abuse).
3. Do **not** add `yggdrasil` to `publishedTo`. At most: “this host is also on Ygg” in ops docs, or a native client that dials our `200::` pin as another provider for the **same** OrbitDB/iroh payload.
4. Stay off the product critical path until yggdrasil-go is out of **alpha** if we would depend on wire compatibility.

---


## Pain points and risks

1. **WASM packaging.** No official browser npm package. `@number0/iroh` is Node NAPI only. Ship a wasm-bindgen wrapper (`default-features = false`). Nuxt bundle size, wasm-pack CI, possible COOP/COEP. Examples: n0 `browser-echo`, `browser-chat`.
2. **Relay on the hot path for every browser byte.** Bandwidth and long-lived WebSockets share the **API process**. Abuse, FD limits, and a dictionary-handler deadlock can starve P2P. Need caps, timeouts, and metrics.
3. **Same-process blast radius.** Killing or OOM-ing Actix takes down the home relay. n0 fallback must stay in the client map.
4. **Actix worker isolation.** Wrong if each worker has its own relay registry.
5. **Snapshot hash changes on edit.** Product “permanent link” needs the discovery pointer (or later `iroh-docs`).
6. **Pin or vanish.** Author closes the tab → blob gone unless this process has pinned it. Pin on publish for logged-in users (quota).
7. **Identity.** One iroh secret in IndexedDB. Losing the profile loses republish rights. Encrypted export **before first publish**.
8. **Not anonymous.** Anyone with the ticket can read; relays see connection metadata.
9. **Sync is one-way import** with an explicit diff, not a bidirectional CRDT vs PostgreSQL.
10. **HTTPS blob fallback** (`GET /api/p2p/blob/:ticket`) is the product working when overlay fails. Plan for it.
11. **Open relay on the main origin.** Other iroh apps can use `/relay` if they know the URL. Caps, monitoring, ToS.
12. **Version lockstep.** WASM crate, Actix `iroh` / `iroh-relay`, and n0 fallback must stay on compatible 1.x.
13. **Legal.** Pinning = we host user content (ToS, takedown, size quotas).
14. **Media.** Keep images on Lensisku HTTPS or separate size-capped blobs; do not inflate the snapshot over the relay.
15. **Safari / background tabs** drop WebSockets. Resume publish/import; do not corrupt IndexedDB.
16. **Reverse proxy.** Missing WebSocket proxy for `/relay` (iroh) or `/p2p-ws` (libp2p) looks like “P2P never works.”
17. **ATProto writes need a live PDS** and OAuth/DPoP in Nuxt (client metadata, refresh, PDS routing).
18. **ATProto records are public** on the firehose. Do not promise private collections on ATProto.
19. **~100 KB record cap** on common PDSes → one record per item, not one blob for the whole collection.
20. **PLC + Bluesky brownouts** block login and new firehose events; our index still serves old ATProto rows.
21. **Relay desync** for self-hosted authors: they published; Lensisku never indexed until recrawl. Show “on PDS but not in catalog” if `getRecord` works and Jetstream did not.
22. **Three identities.** OrbitDB key ≠ iroh secret ≠ ATProto DID. Export all.
23. **Multi-publish drift.** Overlays diverge if the user publishes to only one after an edit. UI shows per-backend freshness vs local `updatedAt`.
24. **ToS.** Rate-limit ATProto item floods on shared PDSes.
25. **Helia + iroh + ATProto** in one SPA: bundle size and three failure modes.
26. **Circuit-relay / TURN** still centralizes Helia when WebRTC fails — run on our origin; admit it is a server.
27. **OrbitDB without a pin:** last peer offline → address does not resolve until someone with blocks returns.
28. **Public IPFS** may cache published blocks; same content warning as any overlay.
29. **Yggdrasil is alpha** (protocol can change); public mesh is a **testnet**; not anonymity; firewall the TUN; distant public peers = you may **transit** others’ traffic.
30. **Ygg in the browser** needs our WSS + CORS; that is another relay-shaped service. Do not treat WASM demos as production Helia replacement.
31. **Private Ygg + one public peer = accidental global bridge.** Document this if we ever ship a “Lensisku mesh” config.

---

## Single-owner collections

One person **writes**. Others **follow** (live OrbitDB replica) or **fork**. They do not merge two writers.

```
Create local → Publish (OrbitDB default) → peer opens address → replica stays live-synced
         → optional iroh / ATProto copies → optional Fork → their own address
```

---

## Data model

### IndexedDB

```typescript
// definitions
{
  localId: string,
  word: string,
  definition: string,
  languageId: number,
  sourceLanguageId: number,
  status: 'local' | 'published' | 'server-synced',
  remoteId?: number,
  createdAt: string,
  updatedAt: string
}

// collections
{
  localId: string,
  name: string,
  description: string,
  visibility: 'public' | 'private',
  status: 'local' | 'published' | 'server-synced',
  blobTicket?: string,
  collectionKey?: string,
  atUri?: string,
  atDid?: string,
  publishedTo: Array<'orbitdb' | 'iroh' | 'atproto'>,
  orbitDbAddress?: string,
  remoteCollectionId?: number,
  itemCount: number,
  createdAt: string,
  updatedAt: string,
  lastSyncedAt?: string,
  lastOrbitDbPublishAt?: string,
  lastIrohPublishAt?: string,
  lastAtprotoPublishAt?: string,
  syncStatus?: 'pending' | 'syncing' | 'synced' | 'conflict'
}

// collectionItems
{
  localId: string,
  collectionLocalId: string,
  definitionLocalId: string,
  order: number,
  addedAt: string
}

// orbitDbRegistry
{
  address: string,             // /orbitdb/zdpu… primary key
  type: 'own' | 'following',
  title: string,
  description: string,
  ownerIdentityId: string,
  itemCount: number,
  lastHead?: string,           // latest log entry hash, for sync UI
  replicationStatus: 'idle' | 'connecting' | 'live' | 'error',
  lastError?: string,
  isPinnedInUi: boolean,
  discoveredVia: 'manual' | 'lensisku-discovery' | 'peer',
  updatedAt: string
}

// orbitIdentity (single row)
{
  publicKey: string,
  encryptedPrivateKey?: string,  // or raw with backup warning
  identityId: string,
  backedUpAt?: string
}

// p2pRegistry (iroh tickets we know)
{
  ticket: string,
  collectionKey?: string,
  type: 'own' | 'subscribed',
  title: string,
  description: string,
  ownerEndpointId: string,
  itemCount: number,
  lastSyncedAt: string,
  isPinned: boolean,
  discoveredVia: 'manual' | 'lensisku-discovery' | 'peer'
}

// atprotoRegistry
{
  atUri: string,
  collectionKey: string,
  did: string,
  handle?: string,
  type: 'own' | 'subscribed',
  title: string,
  itemCount: number,
  lastSyncedAt: string,
  discoveredVia: 'manual' | 'lensisku-discovery'
}

// irohIdentity
{
  secretKey: string,
  endpointId: string,
  backedUpAt?: string
}

// atprotoSession (OAuth tokens; treat as secret)
{
  did: string,
  handle: string,
  pdsUrl: string,
  // refresh material stored per @atproto/oauth-client guidance
}

// syncLog
{
  id: string,
  collectionLocalId: string,
  timestamp: string,
  operation:
    | 'import-from-orbitdb'
    | 'publish-to-orbitdb'
    | 'import-from-iroh'
    | 'import-from-atproto'
    | 'publish-to-iroh'
    | 'publish-to-atproto'
    | 'sync-to-server',
  itemsAdded: number,
  itemsRemoved: number,
  itemsModified: number,
  details: {
    added: Array<{ word: string, localId: string }>,
    removed: Array<{ word: string, localId: string }>,
    modified: Array<{ word: string, localId: string, changes: string[] }>
  }
}
```

### Snapshot JSON (iroh blob)

```json
{
  "schemaVersion": 1,
  "title": "My Lojban Travel Phrases",
  "description": "Useful phrases for traveling",
  "license": "CC-BY-SA 4.0",
  "languageId": "en",
  "sourceLanguageId": "jbo",
  "visibility": "public",
  "createdAt": "2026-08-24T00:00:00Z",
  "updatedAt": "2026-08-24T12:18:05Z",
  "itemCount": 42,
  "ownerEndpointId": "<iroh endpoint id>",
  "lensiskuUsername": "gleki",
  "items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "type": "definition",
      "word": "klama",
      "definition": "x1 comes/goes to destination x2...",
      "languageId": "en",
      "sourceLanguageId": "jbo",
      "tags": ["motion", "beginner"],
      "createdAt": "2026-08-24T00:00:00Z",
      "updatedAt": "2026-08-24T00:00:00Z"
    }
  ]
}
```

Item `id` is stable across republishes so sync diffs match.

### OrbitDB documents (Helia)

One **Documents** database per collection. Default `IPFSAccessController`: **only the creating identity can write**. Followers `open` read-only (writes fail and must be blocked in UI).

**Meta** (`_id`: `__meta__`):

```json
{
  "_id": "__meta__",
  "schemaVersion": 1,
  "title": "My Lojban Travel Phrases",
  "description": "Useful phrases for traveling",
  "license": "CC-BY-SA 4.0",
  "languageId": "en",
  "sourceLanguageId": "jbo",
  "visibility": "public",
  "createdAt": "2026-08-24T00:00:00Z",
  "updatedAt": "2026-08-24T12:18:05Z",
  "itemCount": 42,
  "ownerIdentityId": "02e7247a4c…",
  "lensiskuUsername": "gleki"
}
```

**Item** (`_id` = definition `localId` / stable UUID):

```json
{
  "_id": "550e8400-e29b-41d4-a716-446655440000",
  "type": "definition",
  "word": "klama",
  "definition": "x1 comes/goes to destination x2...",
  "languageId": "en",
  "sourceLanguageId": "jbo",
  "tags": ["motion", "beginner"],
  "createdAt": "2026-08-24T00:00:00Z",
  "updatedAt": "2026-08-24T00:00:00Z"
}
```

Deletes: `db.del(id)` (tombstone in the log). Do not reuse `_id` for a different word.

ATProto item records use the same fields (minus wrapping `items[]`); `$type`: `org.lojban.lensisku.item`. Meta record `$type`: `org.lojban.lensisku.collection`.

---

## UI

### Collections list

```
Collections (/collections)

┌─────────────────────────────────────────────────┐
│ My Collections                                   │
│ [Server]  [Local & published]                    │
│                                                  │
│ □ Server Collection #1                           │
│                                                  │
│ □ Travel Phrases                                 │
│   • OrbitDB (live P2P) · iroh · ATProto          │
│   • [Follow / Fork] [Publish…] [Share]           │
│                                                  │
│ □ Grammar Notes                                  │
│   • iroh only · not on Lensisku server           │
│   • [Publish to ATProto] [Sync to Server] [Share]│
│                                                  │
│ □ Draft: Vocab List                              │
│   • Local only                                   │
│   • [Publish…]  → OrbitDB (default) ± iroh ± ATProto     │
└─────────────────────────────────────────────────┘
```

**Badges:** Server · OrbitDB (live) · iroh · ATProto · Lensisku-synced · Local · Following (read-only)

**Publish modal:** checkboxes **OrbitDB (P2P sync)** (default on) / **iroh** / **ATProto**. Explain: OrbitDB = friends stay updated without our server when WebRTC works; ATProto = not P2P; iroh browsers need a relay.

**Share modal:** OrbitDB address first (`/orbitdb/zdpu…` + `https://<origin>/orbit/view/<urlencoded-address>`), QR of the product URL; then iroh / ATProto if published.

**Follow vs fork:** Opening a peer address is **Follow** (live replica, read-only). **Fork** copies items into a new local collection the user owns.

### Orbit collection view (`/orbit/view/:address`)

```
Travel Phrases                    [Following · live] [Fork to my library]
/orbitdb/zdpuAX1234…
by identity 02e7… · 15 items · updated just now

[Search]  klama — come/go
          tavla — talk

⚠ Read-only. Fork to edit. Connection: WebRTC · relay · reconnecting…
```

Empty / error: invalid address, timeout, “waiting for a peer who has this DB”, identity-lost (owner).

### Local collection editor (OrbitDB-backed)

If `publishedTo` includes `orbitdb` and we hold the write identity: edits write IndexedDB **and** `db.put` / `db.del` immediately (or debounce 300ms). Show “P2P: live” vs “P2P: offline (queued)” if Helia is down.

---

### Sync diff

```
Sync "Travel Phrases" to Lensisku Server

Ticket: blobticket…   Last synced: 2 days ago

✓ Added (3): klama, tavla, citka
✗ Removed (1): broda
⚠ Modified (1): djica (definition changed)

Server collection: "Travel Phrases" (#42)
[Cancel] [Sync to Server]
```

Diff: load items from local IndexedDB (or from iroh blob / ATProto records) vs `GET /collections/:id`; match on item `id`.

### Share

Show **OrbitDB address** first, then iroh ticket and `at://` if present.

⚠ Anyone with the link can read  
⚠ OrbitDB/IPFS: peers and public IPFS may cache blocks; live sync needs a replica or pin  
⚠ iroh: relay metadata; pin cache  
⚠ ATProto: public firehose / PDS  

### Discovery (`/p2p/discover`)

```
Open by /orbitdb/…, blob ticket, at://, or Lensisku URL
Public collections · chip: OrbitDB | iroh | ATProto
```

Register `backend: "orbitdb" | "iroh" | "atproto"`. Paste bypasses catalog.

---

## Rust backend

**iroh:** `iroh`, `iroh-blobs`, `iroh-relay` (`server`), 1.x — same as below.

**ATProto AppView:** HTTP client + Jetstream consumer (e.g. `atrium` / `reqwest` + websocket). Background task: persist cursor, upsert `p2p_collections` for our NSIDs. Resolve DID → PDS for `getRecord` fallback.

```toml
[dependencies]
iroh = "1"
iroh-blobs = "1"
iroh-relay = { version = "1", features = ["server"] }
# ATProto: pin atrium / jetstream crates current at implement time
```

**Startup (same `start_server`):**

1. Build `RelayMap` from this process’s public origin + n0 prod
2. Bind `Endpoint` + blob store (data dir on disk)
3. Construct shared `RelayService` and register Actix routes `/relay`, `/derp` (WebSocket) plus any relay HTTP probe paths iroh expects
4. `web::Data` for `IrohNode` and relay state

Sketch (intent, not frozen API):

```rust
// src/p2p/mod.rs — node + relay + HTTP API

pub struct IrohNode {
    endpoint: iroh::Endpoint,
    // blobs handle
}

impl IrohNode {
    pub async fn new(public_origin: &str) -> anyhow::Result<Self> {
        let ours: iroh::RelayUrl = public_origin.parse()?;
        let map = iroh::RelayMap::from_iter(
            std::iter::once(ours).chain(iroh::defaults::prod::default_relay_map().urls().cloned()),
        );
        let endpoint = iroh::Endpoint::builder()
            .relay_mode(iroh::RelayMode::Custom(map))
            .bind()
            .await?;
        endpoint.online().await;
        Ok(Self { endpoint })
    }
}
```

Relay WebSocket handler: upgrade on `/relay`, handshake via `iroh-relay`, `clients.register(...)`. One `Clients` map for the whole process.

### HTTP API


| Method | Path | Role |
|---|---|---|
| GET | `/api/p2p/libp2p-bootstrap` | Multiaddrs for Helia (same origin `/p2p-ws` + peer id) |
| GET | `/api/p2p/relay-map` | iroh origins (this host + n0) |
| GET | `/api/p2p/atproto-oauth-client-meta` | OAuth client metadata for ATProto login |
| POST | `/api/p2p/register` | Auth: catalog orbitdb address, iroh ticket, and/or `atUri` |
| GET | `/api/p2p/public-collections` | Mixed catalog (`backend` field) |
| GET | `/api/p2p/blob/{ticket}` | iroh HTTPS fallback |
| GET | `/api/p2p/atproto/record` | Query `uri=`; fetch from PDS or our index |
| POST | `/api/p2p/sync-diff` | Snapshot vs server collection |
| POST | `/api/p2p/sync-to-server` | Apply diff |

Writes to ATProto **do not** go through Actix; the browser talks to the PDS. Register may verify `getRecord` so we do not catalog a URI we cannot read.

### SQL

```sql
CREATE TABLE p2p_collections (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    backend TEXT NOT NULL CHECK (backend IN ('orbitdb', 'iroh', 'atproto')),
    blob_ticket TEXT,
    at_uri TEXT,
    orbitdb_address TEXT,
    collection_key TEXT,
    title TEXT NOT NULL,
    description TEXT,
    visibility TEXT NOT NULL CHECK (visibility IN ('public', 'private')),
    owner_endpoint_id TEXT,
    owner_did TEXT,
    item_count INTEGER NOT NULL DEFAULT 0,
    lensisku_collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL,
    last_synced_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_p2p_orbitdb ON p2p_collections(orbitdb_address)
    WHERE orbitdb_address IS NOT NULL;
CREATE UNIQUE INDEX idx_p2p_iroh_ticket ON p2p_collections(blob_ticket)
    WHERE blob_ticket IS NOT NULL;
CREATE UNIQUE INDEX idx_p2p_at_uri ON p2p_collections(at_uri)
    WHERE at_uri IS NOT NULL;

CREATE TABLE p2p_pins (
    blob_ticket TEXT PRIMARY KEY,
    pinned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    pin_status TEXT NOT NULL CHECK (pin_status IN ('pinning', 'pinned', 'failed')),
    blake3_hash TEXT NOT NULL
);

CREATE TABLE p2p_jetstream_cursor (
    service TEXT PRIMARY KEY,
    cursor TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## Frontend — OrbitDB / Helia (full)

pnpm under `frontend/`. Pin versions at implement time (`helia`, `@orbitdb/core`, `@libp2p/*`).

### npm packages

```json
{
  "idb": "^8.0.0",
  "qrcode": "^1.5.3",
  "helia": "^5.0.0",
  "@orbitdb/core": "^2.4.0",
  "blockstore-idb": "^2.0.0",
  "datastore-idb": "^2.0.0",
  "@chainsafe/libp2p-gossipsub": "^14.0.0",
  "@libp2p/identify": "^3.0.0",
  "@libp2p/websockets": "^9.0.0",
  "@libp2p/webrtc": "^5.0.0",
  "@libp2p/circuit-relay-v2": "^3.0.0",
  "@libp2p/bootstrap": "^11.0.0",
  "@multiformats/multiaddr": "^12.0.0"
}
```

Do **not** import iroh WASM into the Helia path. Do **not** use Node-only `@number0/iroh` in Nuxt.

### Files

```
frontend/src/
├── localdb/
│   ├── schema.ts                 — IndexedDB (incl. orbitDbRegistry, orbitIdentity)
│   ├── definitions.ts
│   ├── collections.ts
│   ├── collectionItems.ts
│   ├── orbitRegistry.ts
│   ├── orbitIdentity.ts          — load/save/export/import identity
│   └── syncLog.ts
├── p2p/helia/
│   ├── createHeliaBrowser.ts     — libp2p + Helia singleton factory
│   ├── libp2pConfig.ts           — WebRTC + WS circuit-relay to same origin
│   ├── orbitFactory.ts           — createOrbitDB + Identities keystore
│   └── addresses.ts              — parse /orbitdb/, Lensisku /orbit/view/ URLs
├── composables/
│   ├── useLocalDefinitions.ts
│   ├── useLocalCollections.ts
│   ├── useOrbitDb.ts             — start/stop Helia+OrbitDB (one per tab)
│   ├── useOrbitPublish.ts        — local collection → Documents db
│   ├── useOrbitFollow.ts         — open address, live events → UI / optional IDB
│   ├── useOrbitFork.ts           — copy items to new owned collection
│   ├── useOrbitIdentityBackup.ts
│   ├── useIroh.ts                — separate overlay
│   ├── useAtproto.ts
│   ├── useP2pPublish.ts          — picker: orbitdb ± iroh ± atproto
│   ├── useP2pSync.ts             — diff vs PostgreSQL
│   └── useP2pDiscovery.ts
├── pages/
│   ├── LocalDashboard.vue              — /local
│   ├── LocalCollections.vue            — /local/collections
│   ├── LocalCollectionDetail.vue       — /local/collections/:localId
│   ├── OrbitDiscover.vue               — /orbit/discover (also mixed /p2p/discover)
│   ├── OrbitCollectionView.vue         — /orbit/view/:address
│   ├── P2pCollectionView.vue           — /p2p/view/:ticket (iroh)
│   └── AtprotoCollectionView.vue       — /at/:handleOrDid/:key
├── components/
│   ├── CollectionStatusBadge.vue
│   ├── PublishBackendPicker.vue
│   ├── OrbitShareModal.vue
│   ├── OrbitAddressInput.vue
│   ├── OrbitConnectionStatus.vue       — WebRTC / relay / offline
│   ├── OrbitFollowBanner.vue           — read-only + Fork
│   ├── P2pSyncDiffModal.vue
│   └── IdentityBackupModal.vue
└── router: /orbit/discover, /orbit/view/:address
```

### Helia + libp2p (browser)

- **Listen:** none (or WebRTC listen if the stack supports it). **Dial:** `webrtc`, `wss` to `wss://<lensisku-origin>/p2p-ws` (circuit-relay v2). Same origin as the API — nginx must Upgrade WebSockets for `/p2p-ws` like `/relay`.
- **Bootstrap:** our relay’s peer id (from `GET /api/p2p/libp2p-bootstrap`) plus optional public Helia bootstraps. Do not require public IPFS bootstrap for the happy path.
- **STUN:** public STUN (e.g. Google) so WebRTC works when Lensisku is **down**. Optional TURN on the same host later.
- **Persistence:** `blockstore-idb` + `datastore-idb` so refresh does not drop blocks.
- **Gossipsub** required for OrbitDB replication.
- **Identify** enabled so peers exchange listen addrs.

Sketch (APIs drift; verify against Helia/libp2p at implement time):

```typescript
// p2p/helia/libp2pConfig.ts
export function lensiskuLibp2pOptions(opts: {
  relayMultiaddr: string // /dns4/…/tcp/443/wss/p2p-circuit/p2p/<id> or /dns4/…/wss/p2p-circuit
}) {
  return {
    addresses: { listen: ['/webrtc'] },
    transports: [webSockets(), webRTC()],
    connectionEncrypters: [/* noise */],
    streamMuxers: [/* yamux */],
    services: {
      identify: identify(),
      pubsub: gossipsub({ allowPublishToZeroTopicPeers: true }),
      relay: circuitRelayTransport(),
    },
    peerDiscovery: [bootstrap({ list: [opts.relayMultiaddr] })],
  }
}
```

`createHeliaBrowser`: singleton `Promise<Helia>`. `useOrbitDb().stop()` on `pagehide` / logout: close DBs, stop Helia.

### Identity

OrbitDB 2 **Identities** + keystore in IndexedDB (`orbitIdentity` store). Generate on first publish. **Export JSON** (encrypted with user passphrase) before first publish (`IdentityBackupModal`). Import on a second device to keep the **same** write key (same `/orbitdb/` address remains writable). Losing the key = can still **follow** that address from others, cannot **append**.

Access: create with `AccessController: IPFSAccessController` (default). Followers never get write keys. UI hides edit on `useOrbitFollow`.

### `useOrbitDb`

```typescript
type OrbitDbHandle = {
  helia: Helia
  orbitdb: OrbitDB
  identityId: string
  start: () => Promise<void>
  stop: () => Promise<void>
  connection: Ref<'offline' | 'relay' | 'webrtc' | 'mixed'>
}
```

Start lazily on Publish or Open. One Helia per tab. Reuse across collections.

### `useOrbitPublish`

```
publish(collectionLocalId):
  start orbitdb
  open({ type: 'documents', name: `lensisku-${collectionKey}` })
    // name is local; address is content-addressed from identity+name
  put __meta__
  for each item: put document (_id = localId)
  save orbitDbAddress on collection, publishedTo += 'orbitdb'
  upsert orbitDbRegistry type=own
  optional POST /api/p2p/register { backend: 'orbitdb', orbitdb_address }
  keep db open while the editor is mounted (live for followers)
```

Updates: `put` changed docs, update `__meta__.updatedAt` / `itemCount`. Deletes: `del`. Debounce bursts.

If identity changed (user imported another key), **do not** write to the old address; UI: “This collection was published with a different key. Follow-only or Fork.”

### `useOrbitFollow`

```
open(address):
  validate starts with /orbitdb/
  start orbitdb
  db = await orbitdb.open(address)
  load all() → preview
  db.events.on('update', …) → merge into reactive items
  registry type=following, replicationStatus=live
  optional: mirror into IndexedDB as a followed collection (read-only flag)
```

Timeout: after N seconds with 0 entries, show “waiting for peer” + retry. Connection status from libp2p connection list (WebRTC vs relayed).

### `useOrbitFork`

Copy current items + meta into a **new** `localId` collection (`publishedTo: []`). User publishes to a **new** OrbitDB (their identity). Sync log `import-from-orbitdb`.

### Routes and deep links

| Path | Action |
|---|---|
| `/orbit/discover` | Paste `/orbitdb/…` or catalog |
| `/orbit/view/:address` | `encodeURIComponent` of full address |
| `/orbit/view/:address?preview=1` | No IndexedDB mirror |
| `https://<origin>/orbit/view/…` | Share link (QR) |

Parse pasted Lensisku URLs, raw `/orbitdb/zdpu…`, and `orbitdb://` if we add it.

### Vue pages (behavior)

**OrbitDiscover.vue:** input validation; Open → `/orbit/view/…`; list `GET /api/p2p/public-collections?backend=orbitdb`; chips for other backends.

**OrbitCollectionView.vue:** `useOrbitFollow`; search/filter items; Fork; Share; status badge; do not show edit. If `orbitIdentity` matches `ownerIdentityId`, redirect to local editor instead.

**LocalCollectionDetail.vue:** if owner of published OrbitDB, bind `useOrbitPublish` sync-on-save; `OrbitConnectionStatus`; Share.

**OrbitShareModal.vue:** address, copy, QR (`qrcode` on canvas), warnings (public, cached, follow not merge).

**OrbitConnectionStatus.vue:** `offline` / `connecting` / `relay` / `webrtc` / `error`.

**IdentityBackupModal.vue:** force or strongly prompt on first successful publish.

### i18n

Keys under `orbit.*` and `p2p.*` (discover, follow, fork, waitingForPeer, identityBackup, webrtcHint, relayHint). No “IPFS is permanent” without the pin caveat.

### Tests (frontend)

1. Publish collection → address stored → refresh → still owner, items match IndexedDB.
2. Two browsers (same machine): B opens A’s address → sees items → A adds word → B updates without reload.
3. Kill API process: if WebRTC connected, B still receives A’s next put (STUN-only).
4. Follower cannot `put` (UI + expect OrbitDB to reject).
5. Fork → new address ≠ original; A’s later edits do not change fork.
6. Invalid address, hang timeout, stop Helia on navigate away (no leaked connections).

### iroh / ATProto frontend (not OrbitDB)

Keep `useIroh`, `useAtproto`, `/p2p/view/:ticket`, `/at/…` as **additional** publish targets from `PublishBackendPicker`. They must not share Helia’s libp2p node.

---

## User flows

**P2P exchange and live sync**

1. Alice creates a collection under `/local/collections` (IndexedDB).
2. Publish… → **OrbitDB** on (default). Optional iroh / ATProto checkboxes.
3. `useOrbitPublish`: Helia start, Documents db, meta + items, address in IndexedDB.
4. Share modal: product URL + raw `/orbitdb/…` + QR. Optional register for catalog.
5. Bob pastes URL on `/orbit/discover` → `/orbit/view/…` → `useOrbitFollow`.
6. Alice edits a definition → `put` → Bob’s view updates.
7. Bob **Fork** → own local collection → he can publish a new OrbitDB.

**Owner, second device:** import identity backup → same address stays writable.

**Lensisku down:** existing WebRTC sessions keep syncing; new meetings need STUN (and fail if both NATs need our relay).

**Sync to Lensisku PostgreSQL:** `P2pSyncDiffModal` from local items or from followed replica snapshot.

**Discover:** mixed catalog; paste always works without login.

## Implementation plan

### Phase 1 — IndexedDB + local collections

### Phase 2 — Helia + OrbitDB publish/follow (priority)

Frontend (see **Frontend — OrbitDB / Helia**):

1. pnpm deps; `createHeliaBrowser` + IDB blockstore
2. `useOrbitDb` singleton; identity generate/export
3. `useOrbitPublish` / `useOrbitFollow` / `useOrbitFork`
4. Pages: local editor bind, `/orbit/discover`, `/orbit/view/:address`, share + QR, connection status
5. nginx: WebSocket `/p2p-ws`; `GET /api/p2p/libp2p-bootstrap`
6. Tests: two browsers live update; follower cannot write; fork isolation; **Lensisku killed** + WebRTC still syncs; Helia stop on navigate

Backend for this phase: bootstrap endpoint + circuit-relay on same origin (Rust libp2p or same-host helper). Catalog register can wait until Phase 4.

### Phase 3 — iroh WASM + in-process `/relay` + pin

Backup overlay when Helia cannot connect.

### Phase 4 — Discovery catalog (all backends)

### Phase 5 — PostgreSQL sync-diff

### Phase 6 — ATProto OAuth + AppView (optional store)

### Phase 7 — Pin/durability

Long-lived OrbitDB replica or CID pin so addresses survive empty swarms; iroh pin; ATProto PDS.

### Phase 8 — Errors / docs

Explain federation vs P2P vs DFS vs “mesh IPv6 (Ygg)” in the UI in one sentence each; per-backend “live / needs relay / needs PDS.” Ygg: not a publish checkbox.

### Phase 9 — Yggdrasil (optional, after Helia works)

Only if native users ask for mesh IPv6: sidecar daemon + docs (public peers, firewall). Do not block Phases 1–8 on this. No WASM Ygg in the Nuxt bundle unless a later decision reverses open item 15.

---

## Security and privacy

- OrbitDB, iroh, and ATProto secrets/tokens in IndexedDB; export/backup
- Owner-only writes; followers are read-only replicas
- Warn: OrbitDB/IPFS cache, iroh relays, ATProto firehose; Ygg is not private and is globally reachable once peered
- Circuit-relay and iroh `/relay` on the main origin: connection limits

---

## Open decisions

1. IndexedDB vs OPFS for the iroh secret; ATProto token storage
2. Pin / catalog quotas
3. HTTPS fallbacks always on (recommended)
4. Discoverable automatically on publish vs opt-in (per backend)
5. Server-side edits vs later import (explicit diff only)
6. Media: Lensisku HTTPS vs iroh blobs vs ATProto blobs (PDS blob caps)
7. iroh stable pointer: `collection_key` vs `iroh-docs`
8. iroh `/relay` open vs token
9. Second Lensisku region (still API+relay in that process)
10. iroh tickets with embedded addrs vs n0 PKARR
11. Default publish target: **OrbitDB** (recommended) vs ask every time
12. Link Lensisku account ↔ DID before ATProto listing?
13. Rust libp2p circuit-relay in-process vs proxy to a helper on the same host
14. STUN-only vs TURN on Lensisku origin for Helia WebRTC
15. Yggdrasil: skip (recommended) vs optional native underlay vs WSS peer on origin for WASM (high cost, low SPA benefit)
16. If we bind Actix on Ygg IPv6: same process TUN vs sidecar yggdrasil-go / yggstack; never list ourselves as a **public** peer unless we accept transit

**Next step:** Phase 1, then **Helia/OrbitDB two-browser sync** (including Lensisku-down test). iroh relay and ATProto are extra stores. Yggdrasil is **not** on the MVP path.