# KromaRec 0.10.1

- Password changes and completed RecNet logouts now invalidate active SignalR connections and admitted Game actors. Admission and account invalidation are ordered by the existing SQLite executor. A cancelled HTTP request cannot skip invalidation after its database mutation commits.
- SignalR respects the expiry of the bearer used to open a socket. A stale login lock and the Orientation logout exception retain their existing behavior. Normal logout preserves the existing HTTP login contract.
- The updated Luxon 1.5 Game handler holds a bounded, authenticated loopback connection for each actor, observes backend loss and uses the normal ENet disconnect and actor-leave path. Active reservations and operator SQL bans are rechecked while an actor is connected. There are no database lookups on the event relay path.
- Luxon publishes persistent handoff state when ENet disconnects. The wire harness waits for the disconnect acknowledgement before starting the next hop.
- Discovery and API config responses share immutable response buffers. Opt-in startup diagnostics now include fixed Photon stage labels, admission outcomes, numeric matchmaking results and recognized built-in scene IDs.
- Schema 8, Cargo dependencies, client ticket format and the 689-route inventory are unchanged. Deploy the backend and the new patched Luxon together.
- Regression coverage now exercises live revocation, reconnection, account isolation, expiring SignalR access, room deletion, backend stall and shutdown. Native Windows startup and gameplay remain **UNVERIFIED**.

See [verification](compat/verification-0.10.1.md) and [deployment and two-client acceptance](compat/multiplayer.md).
